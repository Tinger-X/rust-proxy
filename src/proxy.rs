use crate::auth::{check_authentication, AuthConfig};
use crate::connection::{extract_proxy_auth, send_auth_required_response, send_error_response};
use crate::handlers;
use crate::parser::detector::ProtocolType;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct Proxy {
    auth_config: Option<AuthConfig>,
}

impl Proxy {
    pub fn new(auth_config: Option<AuthConfig>) -> Self {
        Self { auth_config }
    }

    pub async fn handle_connection(&self, mut stream: TcpStream, client_addr: SocketAddr) {
        let mut buffer = [0u8; 8192];

        match stream.read(&mut buffer).await {
            Ok(0) => {
                info!("[{}] 客户端关闭连接", client_addr);
                return;
            }
            Ok(n) => {
                let client_addr_str = client_addr.to_string();
                debug!("[{}] 收到 {} 字节数据", client_addr_str, n);

                // 提取认证头
                let auth_header = extract_proxy_auth(&buffer[..n]);

                // 检查认证
                if !check_authentication(&self.auth_config, auth_header.as_deref()) {
                    info!("[{}] 认证失败，需要代理认证", client_addr_str);
                    if let Err(e) = send_auth_required_response(&mut stream).await {
                        error!("[{}] 发送认证要求响应失败: {}", client_addr_str, e);
                    }
                    return;
                }

                // 检测协议类型
                let protocol = crate::parser::detector::detect_protocol(&buffer[..n]);
                info!("[{}] 检测到协议: {:?}", client_addr_str, protocol);

                match protocol {
                    // CONNECT隧道（HTTPS/HTTP/2 over TLS）
                    ProtocolType::ConnectTunnel { host, port } => {
                        Proxy::handle_connect_tunnel(stream, client_addr_str.clone(), host, port)
                            .await;
                    }

                    // HTTP/1.0
                    ProtocolType::Http10 => {
                        if let Err(e) = handlers::http1::handle_http1(
                            stream,
                            client_addr_str.clone(),
                            &self.auth_config,
                            &buffer[..n],
                        )
                        .await
                        {
                            error!("[{}] HTTP/1.0处理失败: {}", client_addr_str, e);
                        }
                    }

                    // HTTP/1.1
                    ProtocolType::Http11 => {
                        if let Err(e) = handlers::http1::handle_http1(
                            stream,
                            client_addr_str.clone(),
                            &self.auth_config,
                            &buffer[..n],
                        )
                        .await
                        {
                            error!("[{}] HTTP/1.1处理失败: {}", client_addr_str, e);
                        }
                    }

                    // HTTP/2 (clear-text)
                    ProtocolType::Http2 => {
                        // HTTP/2需要从Host头获取目标
                        if let Some((host, port)) =
                            crate::connection::parse_http_request(&buffer[..n]).await
                        {
                            if let Err(e) = handlers::http2::handle_http2(
                                stream,
                                client_addr_str.clone(),
                                &host,
                                port,
                                &buffer[..n],
                            )
                            .await
                            {
                                error!("[{}] HTTP/2处理失败: {}", client_addr_str, e);
                            }
                        } else {
                            error!("[{}] HTTP/2请求缺少Host头", client_addr_str);
                            let _ =
                                send_error_response(&mut stream, "400 Bad Request", "缺少Host头")
                                    .await;
                        }
                    }

                    // WebSocket升级
                    ProtocolType::WebSocketUpgrade {
                        key: _,
                        host: _,
                        port: _,
                    } => match crate::handlers::websocket::parse_websocket_upgrade(&buffer[..n]) {
                        Ok(Some(upgrade)) => {
                            if let Err(e) = handlers::websocket::handle_websocket(
                                stream,
                                client_addr_str.clone(),
                                upgrade,
                            )
                            .await
                            {
                                error!("[{}] WebSocket处理失败: {}", client_addr_str, e);
                            }
                        }
                        Ok(None) => {
                            error!("[{}] WebSocket升级请求解析失败", client_addr_str);
                            let _ = send_error_response(
                                &mut stream,
                                "400 Bad Request",
                                "无效的WebSocket升级请求",
                            )
                            .await;
                        }
                        Err(e) => {
                            error!("[{}] WebSocket升级请求解析错误: {}", client_addr_str, e);
                            let _ = send_error_response(
                                &mut stream,
                                "400 Bad Request",
                                "解析WebSocket请求失败",
                            )
                            .await;
                        }
                    },

                    // 未知协议
                    ProtocolType::Unknown => {
                        error!("[{}] 无法识别协议类型", client_addr_str);
                        let _ =
                            send_error_response(&mut stream, "400 Bad Request", "无法识别的协议")
                                .await;
                    }
                }
            }
            Err(e) => {
                error!("[{}] 读取客户端数据失败: {}", client_addr, e);
            }
        }
    }

    /// 处理CONNECT隧道请求（HTTPS/HTTP/2 over TLS）
    async fn handle_connect_tunnel(
        mut stream: TcpStream,
        client_addr: String,
        host: String,
        port: u16,
    ) {
        let client_addr_str = client_addr.to_string();
        info!("[{}] CONNECT隧道到 {}:{}", client_addr_str, host, port);

        // 立即发送连接成功响应
        let response = b"HTTP/1.0 200 Connection Established\r\n\r\n";
        if let Err(e) = stream.write_all(response).await {
            error!("[{}] 发送连接成功响应失败: {}", client_addr_str, e);
            return;
        }

        if let Err(e) = stream.flush().await {
            error!("[{}] 刷新响应失败: {}", client_addr_str, e);
            return;
        }

        info!("[{}] 连接建立成功，开始透明转发", client_addr_str);

        // 连接到目标服务器
        match TcpStream::connect((host.as_str(), port)).await {
            Ok(target_stream) => {
                info!(
                    "[{}] 成功连接到目标服务器 {}:{}",
                    client_addr_str, host, port
                );

                // 建立双向透明转发
                let (mut client_reader, mut client_writer) = stream.into_split();
                let (mut target_reader, mut target_writer) = target_stream.into_split();

                let client_to_target = async {
                    let mut buffer = [0u8; 8192];
                    loop {
                        match client_reader.read(&mut buffer).await {
                            Ok(0) => {
                                debug!("[{}] 隧道客户端流结束", client_addr_str);
                                break;
                            }
                            Ok(n) => {
                                if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                                    error!("[{}] 隧道转发到目标失败: {}", client_addr_str, e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("[{}] 读取隧道客户端数据失败: {}", client_addr_str, e);
                                break;
                            }
                        }
                    }
                };

                let target_to_client = async {
                    let mut buffer = [0u8; 8192];
                    loop {
                        match target_reader.read(&mut buffer).await {
                            Ok(0) => {
                                debug!("[{}] 隧道目标流结束", client_addr_str);
                                break;
                            }
                            Ok(n) => {
                                if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                                    error!("[{}] 隧道转发到客户端失败: {}", client_addr_str, e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("[{}] 读取隧道目标数据失败: {}", client_addr_str, e);
                                break;
                            }
                        }
                    }
                };

                tokio::select! {
                    _ = client_to_target => {
                        debug!("[{}] 隧道客户端到目标连接结束", client_addr_str);
                    }
                    _ = target_to_client => {
                        debug!("[{}] 隧道目标到客户端连接结束", client_addr_str);
                    }
                }
            }
            Err(e) => {
                error!(
                    "[{}] 连接目标服务器失败 {}:{}: {}",
                    client_addr_str, host, port, e
                );
                // 连接失败，连接将被客户端或服务端关闭
            }
        }
    }
}
