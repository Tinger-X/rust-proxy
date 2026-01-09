use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::common::CConfig;
use rust_proxy::auth::AuthConfig;
use rust_proxy::proxy::Proxy;

/// 测试代理服务器实例
#[allow(dead_code)]
pub struct TestProxy {
    config: CConfig::TestProxyConfig,
    _handle: JoinHandle<()>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl TestProxy {
    /// 启动测试代理服务器
    pub async fn start(config: CConfig::TestProxyConfig) -> Self {
        let auth_config = if config.auth_required {
            if let (Some(user), Some(pass)) = (&config.username, &config.password) {
                Some(AuthConfig::new(user.clone(), pass.clone()))
            } else {
                None
            }
        } else {
            None
        };

        let proxy = Proxy::new(auth_config, rust_proxy::config::Config::default());
        let addr = config.address();
        let listener = TcpListener::bind(&addr)
            .await
            .expect(&format!("Failed to bind to {}", addr));

        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, remote_addr)) => {
                                let permit = match semaphore.clone().acquire_owned().await {
                                    Ok(permit) => permit,
                                    Err(_) => continue,
                                };

                                let proxy_clone = proxy.clone();
                                tokio::spawn(async move {
                                    proxy_clone.handle_connection(stream, remote_addr).await;
                                    drop(permit);
                                });
                            }
                            Err(_) => break,
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        // 等待代理启动
        sleep(Duration::from_millis(100)).await;

        TestProxy {
            config,
            _handle: handle,
            shutdown_tx,
        }
    }

    /// 停止代理服务器
    pub async fn stop(self) {
        let _ = self.shutdown_tx.send(());
        self._handle.await.ok();
    }

    /// 获取配置
    #[allow(dead_code)]
    pub fn config(&self) -> &CConfig::TestProxyConfig {
        &self.config
    }

    /// 获取代理地址
    #[allow(dead_code)]
    pub fn address(&self) -> String {
        self.config.address()
    }
    /// 获取端口号
    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.config.port
    }

    /// 获取协议类型
    #[allow(dead_code)]
    pub fn protocol(&self) -> CConfig::ProxyProtocol {
        self.config.protocol
    }

    /// 是否需要认证
    #[allow(dead_code)]
    pub fn auth_required(&self) -> bool {
        self.config.auth_required
    }

    /// 获取用户名（如果有）
    #[allow(dead_code)]
    pub fn username(&self) -> Option<&String> {
        self.config.username.as_ref()
    }

    /// 获取密码（如果有）
    #[allow(dead_code)]
    pub fn password(&self) -> Option<&String> {
        self.config.password.as_ref()
    }

    /// 生成认证头
    pub fn auth_header(&self) -> Option<String> {
        self.config.auth_header()
    }
}
