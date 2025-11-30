use crate::auth::{check_authentication, AuthConfig};
use crate::connection::*;
use std::net::SocketAddr;
use std::time::Duration;
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
        let mut buffer = [0u8; 4096];

        match stream.read(&mut buffer).await {
            Ok(0) => {
                info!("[{}] 客户端关闭连接", client_addr);
                return;
            }
            Ok(n) => {
                debug!("[{}] 收到 {} 字节数据: {}", client_addr, n, String::from_utf8_lossy(&buffer[..n]));
                let auth_header = extract_proxy_auth(&buffer[..n]);

                // 检查认证
                if !check_authentication(&self.auth_config, auth_header.as_deref()) {
                    info!("[{}] 认证失败，需要代理认证", client_addr);
                    if let Err(e) = send_auth_required_response(&mut stream).await {
                        error!("[{}] 发送认证要求响应失败: {}", client_addr, e);
                    }
                    return;
                }

                // 处理CONNECT请求（HTTPS隧道）
                if let Some((host, port)) = parse_connect_request(&buffer[..n]).await {
                    info!("[{}] 收到 CONNECT 请求到 {}:{}", client_addr, host, port);

                    // 先尝试连接目标服务器
                    match TcpStream::connect((host.as_str(), port)).await {
                        Ok(mut target_stream) => {
                            info!("[{}] 成功连接到目标服务器 {}:{}", client_addr, host, port);
                            
                            // 只有成功连接目标服务器后，才发送成功响应
                            // 确保响应格式正确，以CRLF结尾
                            let response = b"HTTP/1.1 200 Connection Established\r\n\r\n";
                            debug!("[{}] 发送响应: {}", client_addr, String::from_utf8_lossy(response));
                            
                            // 使用更短的超时时间来快速检测连接问题
                            let write_future = stream.write_all(response);
                            match tokio::time::timeout(Duration::from_secs(5), write_future).await {
                                Ok(Ok(())) => {
                                    debug!("[{}] 响应写入成功", client_addr);
                                }
                                Ok(Err(e)) => {
                                    error!("[{}] 发送连接成功响应失败: {}", client_addr, e);
                                    return;
                                }
                                Err(_) => {
                                    error!("[{}] 发送响应超时", client_addr);
                                    return;
                                }
                            }
                            
                            // 确保响应被立即发送
                            match tokio::time::timeout(Duration::from_secs(5), stream.flush()).await {
                                Ok(Ok(())) => {
                                    debug!("[{}] 响应刷新成功", client_addr);
                                }
                                Ok(Err(e)) => {
                                    error!("[{}] 刷新响应失败: {}", client_addr, e);
                                    return;
                                }
                                Err(_) => {
                                    error!("[{}] 刷新响应超时", client_addr);
                                    return;
                                }
                            }
                            
                            info!("[{}] 成功发送200 Connection Established响应", client_addr);

                            // 建立双向数据转发
                            let (mut client_reader, mut client_writer) = stream.into_split();
                            let (mut target_reader, mut target_writer) = target_stream.into_split();

                            let client_to_target = async {
                                let mut buffer = [0u8; 4096];
                                loop {
                                    match client_reader.read(&mut buffer).await {
                                        Ok(0) => {
                                            debug!("[{}] 客户端到目标服务器流结束", client_addr);
                                            break;
                                        }
                                        Ok(n) => {
                                            if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                                                error!("[{}] 写入目标服务器失败: {}", client_addr, e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            error!("[{}] 读取客户端数据失败: {}", client_addr, e);
                                            break;
                                        }
                                    }
                                }
                            };

                            let target_to_client = async {
                                let mut buffer = [0u8; 4096];
                                loop {
                                    match target_reader.read(&mut buffer).await {
                                        Ok(0) => {
                                            debug!("[{}] 目标服务器到客户端流结束", client_addr);
                                            break;
                                        }
                                        Ok(n) => {
                                            if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                                                error!("[{}] 写入客户端失败: {}", client_addr, e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            error!("[{}] 读取目标服务器数据失败: {}", client_addr, e);
                                            break;
                                        }
                                    }
                                }
                            };

                            tokio::select! {
                                _ = client_to_target => {
                                    debug!("[{}] 客户端到目标服务器连接结束", client_addr);
                                }
                                _ = target_to_client => {
                                    debug!("[{}] 目标服务器到客户端连接结束", client_addr);
                                }
                            }
                        }
                        Err(e) => {
                            error!("[{}] 连接目标服务器失败 {}:{}: {}", client_addr, host, port, e);
                            // 发送连接失败响应
                            if let Err(send_err) = send_error_response(
                                &mut stream,
                                "502 Bad Gateway",
                                &format!("无法连接到目标服务器 {}:{}", host, port),
                            ).await {
                                error!("[{}] 发送错误响应失败: {}", client_addr, send_err);
                            }
                        }
                    }
                    return;
                }

                // 处理HTTP请求
                if let Some((host, port)) = parse_http_request(&buffer[..n]).await {
                    info!("[{}] 收到 HTTP 请求到 {}:{}", client_addr, host, port);

                    match TcpStream::connect((host.as_str(), port)).await {
                        Ok(mut target_stream) => {
                            // 转发原始请求
                            if let Err(e) = target_stream.write_all(&buffer[..n]).await {
                                error!("[{}] 转发HTTP请求失败: {}", client_addr, e);
                                return;
                            }

                            // 建立双向转发
                            if let Err(e) = handle_client(stream, client_addr, &host, port).await {
                                error!("[{}] 处理客户端连接失败: {}", client_addr, e);
                            }
                        }
                        Err(e) => {
                            error!("[{}] 连接目标服务器失败 {}:{}: {}", client_addr, host, port, e);
                            if let Err(send_err) = send_error_response(
                                &mut stream,
                                "502 Bad Gateway",
                                "无法连接到目标服务器",
                            )
                            .await
                            {
                                error!("[{}] 发送错误响应失败: {}", client_addr, send_err);
                            }
                        }
                    }
                } else {
                    error!("[{}] 无法解析请求", client_addr);
                    if let Err(e) =
                        send_error_response(&mut stream, "400 Bad Request", "无法解析请求").await
                    {
                        error!("[{}] 发送错误响应失败: {}", client_addr, e);
                    }
                }
            }
            Err(e) => {
                error!("[{}] 读取客户端数据失败: {}", client_addr, e);
            }
        }
    }
}
