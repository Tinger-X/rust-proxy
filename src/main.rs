use rust_proxy::auth::AuthConfig;
use rust_proxy::config::Config;
use rust_proxy::proxy::Proxy;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn std_main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 解析命令行参数
    let config = Config::from_args();

    // 创建认证配置
    let auth_config = if config.auth_enabled() {
        Some(AuthConfig::new(
            config.username.clone().unwrap(),
            config.password.clone().unwrap(),
        ))
    } else {
        None
    };

    // 创建代理服务器
    let proxy = Proxy::new(auth_config);
    let addr = SocketAddr::new(config.ip, config.port);
    // 绑定监听端口
    let listener = TcpListener::bind(addr).await?;

    if config.auth_enabled() {
        info!(
            "🔒 代理服务器: {}:{} (最大连接数: {})",
            config.ip, config.port, config.max_connections
        );
    } else {
        info!(
            "🔓 代理服务器: {}:{} (最大连接数: {})",
            config.ip, config.port, config.max_connections
        );
    }

    // 创建信号量来限制并发连接数
    let semaphore = Arc::new(Semaphore::new(config.max_connections));

    loop {
        match listener.accept().await {
            Ok((stream, remote_addr)) => {
                info!("接受新连接来自: {}", remote_addr);

                // 获取信号量许可
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(e) => {
                        error!("获取连接许可失败: {}", e);
                        continue;
                    }
                };

                let proxy_clone = proxy.clone();
                tokio::spawn(async move {
                    proxy_clone.handle_connection(stream, remote_addr).await;
                    // 释放许可
                    drop(permit);
                });
            }
            Err(e) => {
                error!("接受连接失败: {}", e);
            }
        }
    }
}

fn main() {
    if let Err(e) = std_main() {
        eprintln!("服务器启动失败: {}", e);
        std::process::exit(1);
    }
}
