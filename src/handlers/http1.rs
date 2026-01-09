use super::backend::BackendConnector;
use crate::connection::send_error_response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info};

/// HTTP/1.x 请求详细信息
pub struct HttpRequest {
    pub host: String,
    pub port: u16,
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// 处理HTTP/1.0和HTTP/1.1请求
pub async fn handle_http1(
    mut client_stream: TcpStream,
    client_addr: String,
    _auth_config: &Option<crate::auth::AuthConfig>,
    buffer: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 解析HTTP请求
    let request = match parse_http_request(buffer) {
        Some(req) => req,
        None => {
            error!("[{}] 无法解析HTTP请求", client_addr);
            // 确定响应的HTTP版本
            let version = if buffer.windows(8).any(|w| w == b"HTTP/1.0") {
                "HTTP/1.0"
            } else {
                "HTTP/1.1"
            };
            send_error_response(&mut client_stream, "400 Bad Request", "无效的HTTP请求", version).await?;
            return Ok(());
        }
    };

    info!(
        "[{}] HTTP/1.x 请求: {} {}://{}:{}{}",
        client_addr,
        request.method,
        if request.port == 443 { "https" } else { "http" },
        request.host,
        request.port,
        request.path
    );

    // 连接到目标服务器
    match BackendConnector::connect(&request.host, request.port).await {
        Ok(target_stream) => {
            debug!(
                "[{}] 成功连接到目标服务器 {}:{}",
                client_addr, request.host, request.port
            );

            // 转发原始请求
            if let Err(e) =
                forward_http_request(client_stream, target_stream, buffer, &client_addr).await
            {
                error!("[{}] HTTP/1.x转发失败: {}", client_addr, e);
            }

            Ok(())
        }
        Err(e) => {
            error!(
                "[{}] 连接目标服务器失败 {}:{}: {}",
                client_addr, request.host, request.port, e
            );
            // 确定响应的HTTP版本
            let version = if buffer.windows(8).any(|w| w == b"HTTP/1.0") {
                "HTTP/1.0"
            } else {
                "HTTP/1.1"
            };
            send_error_response(
                &mut client_stream,
                "502 Bad Gateway",
                &format!("无法连接到 {}:{}", request.host, request.port),
                version,
            )
            .await?;
            Ok(())
        }
    }
}

/// 转发HTTP请求并建立双向数据传输
async fn forward_http_request(
    client_stream: TcpStream,
    target_stream: TcpStream,
    initial_buffer: &[u8],
    client_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 先发送初始缓冲区（HTTP请求）
    let (mut client_reader, mut client_writer) = client_stream.into_split();
    let (mut target_reader, mut target_writer) = target_stream.into_split();

    // 发送初始请求到目标服务器
    target_writer.write_all(initial_buffer).await?;
    debug!("[{}] HTTP请求已转发到目标服务器", client_addr);

    // 双向转发
    let client_to_target = async {
        let mut buffer = [0u8; 8192];
        loop {
            match client_reader.read(&mut buffer).await {
                Ok(0) => {
                    debug!("[{}] HTTP客户端到目标流结束", client_addr);
                    break;
                }
                Ok(n) => {
                    if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                        error!("[{}] HTTP客户端到目标写入失败: {}", client_addr, e);
                        break;
                    }
                }
                Err(e) => {
                    error!("[{}] 读取HTTP客户端数据失败: {}", client_addr, e);
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
                    debug!("[{}] HTTP目标到客户端流结束", client_addr);
                    break;
                }
                Ok(n) => {
                    if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                        error!("[{}] HTTP目标到客户端写入失败: {}", client_addr, e);
                        break;
                    }
                }
                Err(e) => {
                    error!("[{}] 读取HTTP目标数据失败: {}", client_addr, e);
                    break;
                }
            }
        }
    };

    tokio::select! {
        _ = client_to_target => {
            debug!("[{}] HTTP客户端到目标连接结束", client_addr);
        }
        _ = target_to_client => {
            debug!("[{}] HTTP目标到客户端连接结束", client_addr);
        }
    }

    Ok(())
}

/// 解析HTTP请求
fn parse_http_request(buffer: &[u8]) -> Option<HttpRequest> {
    let request = String::from_utf8_lossy(buffer);
    let lines: Vec<&str> = request.lines().collect();

    if lines.is_empty() {
        return None;
    }

    // 解析请求行
    let first_line = lines[0].trim();
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0].to_string();
    let full_path = parts[1].to_string();

    // 提取Host头
    let mut host = String::new();
    let mut port = 80u16;

    for line in &lines {
        if line.to_lowercase().starts_with("host:") {
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
            break;
        }
    }

    // 解析路径（如果有完整URL则提取路径部分）
    let path = if full_path.starts_with("http://") || full_path.starts_with("https://") {
        let start_idx = if full_path.starts_with("http://") {
            7
        } else {
            8
        };
        if let Some(slash_pos) = full_path[start_idx..].find('/') {
            full_path[start_idx + slash_pos..].to_string()
        } else {
            "/".to_string()
        }
    } else {
        full_path.clone()
    };

    // 解析头部
    let mut headers = Vec::new();
    for line in &lines[1..] {
        if line.is_empty() {
            break;
        }
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.push((key, value));
        }
    }

    // 提取body
    let empty_line_idx = lines.iter().position(|l| l.is_empty())?;
    let body_start = request
        .lines()
        .take(empty_line_idx + 1)
        .map(|l| l.len() + 2)
        .sum::<usize>();
    let body: Vec<u8> = buffer[body_start..].to_vec();

    Some(HttpRequest {
        host,
        port,
        method,
        path,
        headers,
        body,
    })
}
