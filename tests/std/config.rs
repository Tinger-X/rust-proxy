use anyhow::{Context, Result};
use base64::prelude::*;
use reqwest::Proxy;

/// 代理服务器配置
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// 代理服务器 IP 地址
    pub ip: String,
    /// 代理服务器端口
    pub port: u16,
    /// 代理认证用户名（可选）
    pub username: Option<String>,
    /// 代理认证密码（可选）
    pub password: Option<String>,
}

impl ProxyConfig {
    /// 从环境变量加载代理配置
    ///
    /// 从 `.env` 文件读取以下环境变量：
    /// - PROXY_IP: 代理服务器 IP 地址
    /// - PROXY_PORT: 代理服务器端口
    /// - PROXY_USERNAME: 代理认证用户名（可选）
    /// - PROXY_PASSWORD: 代理认证密码（可选）
    pub fn from_env() -> Result<Self> {
        // 加载 .env 文件
        dotenv::from_path("tests/std/.env").ok();

        // 读取 IP 地址
        let ip = std::env::var("PROXY_IP")
            .context("PROXY_IP 环境变量未设置，请在 .env 文件中配置代理服务器 IP 地址")?;

        // 读取端口号
        let port_str = std::env::var("PROXY_PORT")
            .context("PROXY_PORT 环境变量未设置，请在 .env 文件中配置代理服务器端口")?;
        let port = port_str
            .parse::<u16>()
            .context("PROXY_PORT 必须是一个有效的端口号（0-65535）")?;

        // 读取用户名（可选）
        let username = std::env::var("PROXY_USERNAME").ok();
        let username = if username.as_ref().map_or(false, |s| !s.is_empty()) {
            Some(username.unwrap())
        } else {
            None
        };

        // 读取密码（可选）
        let password = std::env::var("PROXY_PASSWORD").ok();
        let password = if password.as_ref().map_or(false, |s| !s.is_empty()) {
            Some(password.unwrap())
        } else {
            None
        };

        Ok(ProxyConfig {
            ip,
            port,
            username,
            password,
        })
    }

    /// 获取代理服务器的完整地址（IP:PORT 格式）
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    /// 获取代理服务器的 URL 格式
    pub fn url(&self) -> String {
        format!("http://{}", self.address())
    }

    /// 获取带有认证信息的代理 URL
    ///
    /// 如果配置了用户名和密码，返回格式为：
    /// http://username:password@ip:port
    ///
    /// 如果没有配置认证信息，返回格式为：
    /// http://ip:port
    pub fn proxy_url(&self) -> String {
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => format!("http://{}:{}@{}", user, pass, self.address()),
            _ => self.url(),
        }
    }

    /// 检查是否需要认证
    pub fn requires_auth(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }

    /// 创建 reqwest::Proxy 实例
    pub fn to_proxy(&self) -> Result<Proxy> {
        let proxy_url = self.proxy_url();
        Proxy::all(&proxy_url)
            .with_context(|| format!("无法创建代理客户端，代理 URL: {}", proxy_url))
    }

    /// 生成 Basic 认证头
    ///
    /// 返回 Base64 编码的认证头字符串，格式为：
    /// "Basic <base64_encoded_credentials>"
    pub fn auth_header(&self) -> Option<String> {
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            let credentials = format!("{}:{}", user, pass);
            let encoded = base64::prelude::BASE64_STANDARD.encode(credentials);
            Some(format!("Basic {}", encoded))
        } else {
            None
        }
    }
}
