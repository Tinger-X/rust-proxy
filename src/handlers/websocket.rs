use super::backend::BackendConnector;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info};

/// WebSocket升级请求详细信息
pub struct WebSocketUpgrade {
    pub key: String,
    pub host: String,
    pub port: u16,
    pub path: String,
}

/// 处理WebSocket连接升级和代理
pub async fn handle_websocket(
    mut client_stream: TcpStream,
    client_addr: String,
    upgrade: WebSocketUpgrade,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "[{}] WebSocket升级请求: {}:{}{}",
        client_addr, upgrade.host, upgrade.port, upgrade.path
    );

    // 连接到目标服务器
    match BackendConnector::connect(&upgrade.host, upgrade.port).await {
        Ok(mut target_stream) => {
            debug!(
                "[{}] 成功连接到WebSocket目标服务器 {}:{}",
                client_addr, upgrade.host, upgrade.port
            );

            // 转发原始升级请求到目标服务器
            let upgrade_request = format!(
                "GET {} HTTP/1.1\r\n\
                 Host: {}:{}\r\n\
                 Upgrade: websocket\r\n\
                 Connection: Upgrade\r\n\
                 Sec-WebSocket-Key: {}\r\n\
                 Sec-WebSocket-Version: 13\r\n\
                 \r\n",
                upgrade.path, upgrade.host, upgrade.port, upgrade.key
            );

            if let Err(e) = target_stream.write_all(upgrade_request.as_bytes()).await {
                error!("[{}] 发送WebSocket升级请求失败: {}", client_addr, e);
                send_websocket_error(&mut client_stream, "502 Bad Gateway").await?;
                return Err(e.into());
            }

            debug!("[{}] WebSocket升级请求已发送，等待目标响应", client_addr);

            // 读取目标服务器的升级响应
            let mut response_buffer = [0u8; 4096];
            let n = match target_stream.read(&mut response_buffer).await {
                Ok(0) => {
                    error!("[{}] 目标服务器关闭连接", client_addr);
                    send_websocket_error(&mut client_stream, "502 Bad Gateway").await?;
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => {
                    error!("[{}] 读取目标响应失败: {}", client_addr, e);
                    send_websocket_error(&mut client_stream, "502 Bad Gateway").await?;
                    return Err(e.into());
                }
            };

            let response = String::from_utf8_lossy(&response_buffer[..n]);

            // 检查目标服务器是否同意升级
            if !response.contains("HTTP/1.1 101") && !response.contains("HTTP/1.0 101") {
                error!(
                    "[{}] 目标服务器拒绝WebSocket升级: {}",
                    client_addr,
                    response.lines().next().unwrap_or("未知响应")
                );

                // 将错误响应转发给客户端
                if let Err(e) = client_stream.write_all(&response_buffer[..n]).await {
                    error!("[{}] 转发错误响应失败: {}", client_addr, e);
                }
                return Ok(());
            }

            debug!(
                "[{}] 目标服务器接受WebSocket升级，转发响应给客户端",
                client_addr
            );

            // 转发升级响应给客户端
            if let Err(e) = client_stream.write_all(&response_buffer[..n]).await {
                error!("[{}] 发送WebSocket升级响应失败: {}", client_addr, e);
                return Err(e.into());
            }

            if let Err(e) = client_stream.flush().await {
                error!("[{}] 刷新WebSocket升级响应失败: {}", client_addr, e);
                return Err(e.into());
            }

            debug!("[{}] WebSocket连接建立成功，开始透明转发", client_addr);

            // 建立双向透明转发
            let (mut client_reader, mut client_writer) = client_stream.into_split();
            let (mut target_reader, mut target_writer) = target_stream.into_split();

            let client_to_target = async {
                let mut buffer = [0u8; 8192];
                loop {
                    match client_reader.read(&mut buffer).await {
                        Ok(0) => {
                            debug!("[{}] WebSocket客户端流结束", client_addr);
                            break;
                        }
                        Ok(n) => {
                            if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                                error!("[{}] WebSocket转发到目标失败: {}", client_addr, e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("[{}] 读取WebSocket客户端数据失败: {}", client_addr, e);
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
                            debug!("[{}] WebSocket目标流结束", client_addr);
                            break;
                        }
                        Ok(n) => {
                            if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                                error!("[{}] WebSocket转发到客户端失败: {}", client_addr, e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("[{}] 读取WebSocket目标数据失败: {}", client_addr, e);
                            break;
                        }
                    }
                }
            };

            tokio::select! {
                _ = client_to_target => {
                    debug!("[{}] WebSocket客户端到目标连接结束", client_addr);
                }
                _ = target_to_client => {
                    debug!("[{}] WebSocket目标到客户端连接结束", client_addr);
                }
            }

            Ok(())
        }
        Err(e) => {
            error!(
                "[{}] WebSocket连接目标失败 {}:{}: {}",
                client_addr, upgrade.host, upgrade.port, e
            );
            send_websocket_error(&mut client_stream, "502 Bad Gateway").await?;
            Err(format!("Connection failed: {}", e).into())
        }
    }
}

/// 解析WebSocket升级请求
pub fn parse_websocket_upgrade(
    buffer: &[u8],
) -> Result<Option<WebSocketUpgrade>, Box<dyn std::error::Error + Send + Sync>> {
    let request = String::from_utf8_lossy(buffer);
    let lines: Vec<&str> = request.lines().collect();

    if lines.is_empty() {
        return Ok(None);
    }

    // 解析请求行获取路径
    let first_line = lines[0].trim();
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(None);
    }
    let path = parts[1].to_string();

    // 提取必需的头部
    let mut key = None;
    let mut host = String::new();
    let mut port = 80u16;

    for line in &lines {
        let line_lower = line.to_lowercase();

        if line_lower.starts_with("sec-websocket-key:") {
            key = Some(line[18..].trim().to_string());
        } else if line_lower.starts_with("host:") {
            let host_value = line[5..].trim();
            if let Some(colon_pos) = host_value.find(':') {
                host = host_value[..colon_pos].to_string();
                port = host_value[colon_pos + 1..]
                    .trim()
                    .parse::<u16>()
                    .unwrap_or(80);
            } else {
                host = host_value.to_string();
            }
        }
    }

    // 验证必需字段
    let key = key.ok_or("Missing WebSocket key")?;
    if host.is_empty() {
        return Ok(None);
    }

    Ok(Some(WebSocketUpgrade {
        key,
        host,
        port,
        path,
    }))
}

/// 发送WebSocket错误响应
async fn send_websocket_error(
    stream: &mut TcpStream,
    status: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: text/plain\r\n\
         Connection: close\r\n\
         \r\n",
        status
    );
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}
