use std::error::Error;
use tokio::net::TcpStream;
use tracing::{debug, info};

/// 后端连接器
///
/// 负责使用代理IP连接到目标服务器，确保客户端IP匿名性
pub struct BackendConnector;

impl BackendConnector {
    /// 连接到目标服务器
    ///
    /// # 参数
    /// * `host` - 目标主机名
    /// * `port` - 目标端口
    ///
    /// # 返回
    /// 返回与目标服务器的TCP连接
    pub async fn connect(host: &str, port: u16) -> Result<TcpStream, Box<dyn Error + Send + Sync>> {
        debug!("连接到目标服务器 {}:{}", host, port);

        let stream = TcpStream::connect((host, port)).await?;

        info!("成功连接到目标服务器 {}:{}", host, port);

        Ok(stream)
    }
}
