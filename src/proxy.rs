use crate::auth::{check_authentication, AuthConfig};
use crate::connection::{extract_proxy_auth, send_auth_required_response, send_error_response};
use crate::config::Config;
use crate::handlers;
use crate::parser::detector::ProtocolType;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct Proxy {
    auth_config: Option<AuthConfig>,
    config: Config,
}

impl Proxy {
    pub fn new(auth_config: Option<AuthConfig>, config: Config) -> Self {
        Self { auth_config, config }
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
                        self.handle_connect_tunnel(stream, client_addr_str.clone(), host, port)
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
        &self,
        mut stream: TcpStream,
        client_addr: String,
        host: String,
        port: u16,
    ) {
        let client_addr_str = client_addr.to_string();
        info!("[{}] CONNECT隧道到 {}:{}", client_addr_str, host, port);

        // 设置TCP套接字选项以提高Linux系统上的稳定性
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            use nix::sys::socket::setsockopt;
            use nix::sys::socket::sockopt::TcpNoDelay;
            use nix::sys::socket::sockopt::SoKeepAlive;
            
            if let Ok(fd) = stream.as_raw_fd().try_into() {
                // 禁用Nagle算法，减少延迟（对代理服务器有利）
                let _ = setsockopt(fd, TcpNoDelay, &true);
                // 启用TCP keepalive，检测无效连接
                let _ = setsockopt(fd, SoKeepAlive, &true);
            }
        }

        // 设置连接超时
        let connect_timeout = Duration::from_secs(self.config.connect_timeout);
        
        // 先连接到目标服务器，成功后再发送响应
        match timeout(connect_timeout, TcpStream::connect((host.as_str(), port))).await {
            Ok(Ok(mut target_stream)) => {
                // 为目标连接也设置TCP选项
                #[cfg(unix)]
                {
                    use std::os::unix::io::AsRawFd;
                    use nix::sys::socket::setsockopt;
                    use nix::sys::socket::sockopt::TcpNoDelay;
                    use nix::sys::socket::sockopt::SoKeepAlive;
                    
                    if let Ok(fd) = target_stream.as_raw_fd().try_into() {
                        let _ = setsockopt(fd, TcpNoDelay, &true);
                        let _ = setsockopt(fd, SoKeepAlive, &true);
                    }
                }

                info!(
                    "[{}] 成功连接到目标服务器 {}:{}",
                    client_addr_str, host, port
                );

                // 发送连接成功响应 - 优化发送策略
                let response = b"HTTP/1.0 200 Connection Established\r\n\r\n";
                let write_timeout = Duration::from_secs(self.config.write_timeout);
                
                match timeout(write_timeout, async {
                    let result = stream.write_all(response).await;
                    if result.is_ok() {
                        stream.flush().await
                    } else {
                        result
                    }
                }).await {
                    Ok(Ok(_)) => {
                        info!("[{}] 连接建立成功，开始透明转发", client_addr_str);
                    }
                    Ok(Err(e)) => {
                        debug!("[{}] 发送连接成功响应失败: {}", client_addr_str, e);
                        // 分析错误类型，对于某些常见的Linux网络错误，尝试继续
                        debug!("[{}] 响应发送错误类型: {:?}", client_addr_str, e.kind());
                        info!("[{}] 响应发送失败，但尝试继续透明转发", client_addr_str);
                    }
                    Err(_) => {
                        debug!("[{}] 发送连接成功响应超时", client_addr_str);
                        info!("[{}] 响应发送超时，但尝试继续透明转发", client_addr_str);
                    }
                }

                // 建立双向透明转发
                let (mut client_reader, mut client_writer) = stream.into_split();
                let (mut target_reader, mut target_writer) = target_stream.into_split();

                // 优化双向转发逻辑，添加超时处理和更健壮的错误处理
                let read_timeout = Duration::from_secs(self.config.read_timeout);
                let write_timeout = Duration::from_secs(self.config.write_timeout);

                let client_to_target = async {
                    let mut buffer = [0u8; 8192];
                    loop {
                        match timeout(read_timeout, client_reader.read(&mut buffer)).await {
                            Ok(Ok(0)) => {
                                debug!("[{}] 隧道客户端流结束", client_addr_str);
                                break;
                            }
                            Ok(Ok(n)) => {
                                match timeout(write_timeout, target_writer.write_all(&buffer[..n])).await {
                                    Ok(Ok(_)) => {
                                        // 写入成功
                                    }
                                    Ok(Err(e)) => {
                                        debug!("[{}] 隧道转发到目标失败: {}", client_addr_str, e);
                                        // 尝试刷新缓冲区并继续
                                        let _ = target_writer.flush().await;
                                        continue;
                                    }
                                    Err(_) => {
                                        debug!("[{}] 隧道转发到目标超时", client_addr_str);
                                        continue;
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                debug!("[{}] 读取隧道客户端数据失败: {}", client_addr_str, e);
                                // 对于某些错误，如Connection reset by peer，尝试继续
                                debug!("[{}] 读取错误类型: {:?}", client_addr_str, e.kind());
                                continue;
                            }
                            Err(_) => {
                                debug!("[{}] 读取隧道客户端数据超时", client_addr_str);
                                continue;
                            }
                        }
                    }
                };

                let target_to_client = async {
                    let mut buffer = [0u8; 8192];
                    loop {
                        match timeout(read_timeout, target_reader.read(&mut buffer)).await {
                            Ok(Ok(0)) => {
                                debug!("[{}] 隧道目标流结束", client_addr_str);
                                break;
                            }
                            Ok(Ok(n)) => {
                                match timeout(write_timeout, client_writer.write_all(&buffer[..n])).await {
                                    Ok(Ok(_)) => {
                                        // 写入成功
                                    }
                                    Ok(Err(e)) => {
                                        debug!("[{}] 隧道转发到客户端失败: {}", client_addr_str, e);
                                        let _ = client_writer.flush().await;
                                        continue;
                                    }
                                    Err(_) => {
                                        debug!("[{}] 隧道转发到客户端超时", client_addr_str);
                                        continue;
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                debug!("[{}] 读取隧道目标数据失败: {}", client_addr_str, e);
                                debug!("[{}] 读取错误类型: {:?}", client_addr_str, e.kind());
                                continue;
                            }
                            Err(_) => {
                                debug!("[{}] 读取隧道目标数据超时", client_addr_str);
                                continue;
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
            Ok(Err(e)) => {
                error!(
                    "[{}] 连接目标服务器失败 {}:{}: {}",
                    client_addr_str, host, port, e
                );
                // 连接失败，连接将被客户端或服务端关闭
            }
            Err(_) => {
                error!(
                    "[{}] 连接目标服务器超时 {}:{} ({}秒)",
                    client_addr_str, host, port, self.config.connect_timeout
                );
                let _ = send_error_response(
                    &mut stream,
                    "504 Gateway Timeout",
                    "连接目标服务器超时"
                ).await;
            }
        }
    }
}
