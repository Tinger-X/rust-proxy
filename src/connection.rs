use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info};

pub async fn handle_client(
    client_stream: TcpStream,
    client_addr: SocketAddr,
    target_host: &str,
    target_port: u16,
) -> Result<(), Box<dyn Error>> {
    match TcpStream::connect((target_host, target_port)).await {
        Ok(target_stream) => {
            info!(
                "[{}] 成功连接到目标服务器 {}:{}",
                client_addr, target_host, target_port
            );

            let (mut client_reader, mut client_writer) = client_stream.into_split();
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
            error!(
                "[{}] 连接目标服务器失败 {}: {}: {}",
                client_addr, target_host, target_port, e
            );
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn parse_connect_request(buffer: &[u8]) -> Option<(String, u16)> {
    let request = String::from_utf8_lossy(buffer);
    let lines: Vec<&str> = request.lines().collect();

    if lines.is_empty() {
        return None;
    }

    let first_line = lines[0].trim();
    if !first_line.starts_with("CONNECT ") {
        return None;
    }

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let host_port = parts[1];
    let mut parts = host_port.split(':');
    let host = parts.next()?.to_string();
    let port = parts.next()?.parse::<u16>().ok()?;

    Some((host, port))
}

pub async fn parse_http_request(buffer: &[u8]) -> Option<(String, u16)> {
    let request = String::from_utf8_lossy(buffer);

    if let Some(start) = request.find("Host: ") {
        let host_start = start + 6;
        if let Some(end) = request[host_start..].find('\r') {
            let host_line = &request[host_start..host_start + end];
            let mut parts = host_line.split(':');
            let host = parts.next()?.to_string();
            let port = if let Some(port_str) = parts.next() {
                port_str.parse::<u16>().ok()
            } else {
                Some(80)
            };

            return port.map(|p| (host, p));
        }
    }

    None
}

pub fn extract_proxy_auth(buffer: &[u8]) -> Option<String> {
    let request = String::from_utf8_lossy(buffer);

    if let Some(start) = request.find("Proxy-Authorization: ") {
        let auth_start = start + 21;
        if let Some(end) = request[auth_start..].find('\r') {
            return Some(request[auth_start..auth_start + end].to_string());
        }
    }

    None
}

pub async fn send_auth_required_response(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let response = "HTTP/1.0 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"RustProxy\"\r\n\r\n";
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn send_error_response(
    stream: &mut TcpStream,
    status: &str,
    message: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let response = format!(
        "HTTP/1.0 {}\r\nContent-Type: text/plain\r\n\r\n{}\r\n",
        status, message
    );
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}
