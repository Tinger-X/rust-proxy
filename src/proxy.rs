use crate::auth::{check_authentication, AuthConfig};
use crate::connection::*;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info};

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

                    let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        error!("[{}] 发送连接成功响应失败: {}", client_addr, e);
                        return;
                    }

                    if let Err(e) = handle_client(stream, client_addr, &host, port).await {
                        error!("[{}] 处理客户端连接失败: {}", client_addr, e);
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
