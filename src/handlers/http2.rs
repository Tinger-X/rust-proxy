use super::backend::BackendConnector;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info};

/// 处理HTTP/2连接
///
/// HTTP/2 clear-text模式：直接转发数据流
/// 注意：HTTP/2 over TLS需要通过CONNECT隧道处理
pub async fn handle_http2(
    mut client_stream: TcpStream,
    client_addr: String,
    host: &str,
    port: u16,
    initial_buffer: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("[{}] HTTP/2 连接到 {}:{}", client_addr, host, port);

    // 连接到目标服务器
    match BackendConnector::connect(host, port).await {
        Ok(mut target_stream) => {
            debug!("[{}] 成功建立HTTP/2后端连接", client_addr);

            // 先转发初始缓冲区（包含HTTP/2 preface）
            if let Err(e) = target_stream.write_all(initial_buffer).await {
                error!("[{}] 写入HTTP/2 preface失败: {}", client_addr, e);
                return Err(e.into());
            }

            // 双向转发HTTP/2数据流
            let (mut client_reader, mut client_writer) = client_stream.into_split();
            let (mut target_reader, mut target_writer) = target_stream.into_split();

            let client_to_target = async {
                let mut buffer = [0u8; 8192];
                loop {
                    match client_reader.read(&mut buffer).await {
                        Ok(0) => {
                            debug!("[{}] HTTP/2客户端流结束", client_addr);
                            break;
                        }
                        Ok(n) => {
                            if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                                error!("[{}] HTTP/2转发到目标失败: {}", client_addr, e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("[{}] 读取HTTP/2客户端数据失败: {}", client_addr, e);
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
                            debug!("[{}] HTTP/2目标流结束", client_addr);
                            break;
                        }
                        Ok(n) => {
                            if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                                error!("[{}] HTTP/2转发到客户端失败: {}", client_addr, e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("[{}] 读取HTTP/2目标数据失败: {}", client_addr, e);
                            break;
                        }
                    }
                }
            };

            tokio::select! {
                _ = client_to_target => {
                    debug!("[{}] HTTP/2客户端到目标连接结束", client_addr);
                }
                _ = target_to_client => {
                    debug!("[{}] HTTP/2目标到客户端连接结束", client_addr);
                }
            }

            Ok(())
        }
        Err(e) => {
            error!(
                "[{}] HTTP/2连接目标失败 {}:{}: {}",
                client_addr, host, port, e
            );

            // 返回HTTP/1.1错误响应
            let error_response = b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n";
            let _ = client_stream.write_all(error_response).await;

            Err(format!("Connection failed: {}", e).into())
        }
    }
}
