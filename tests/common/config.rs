use base64::prelude::*;

/// 测试代理协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyProtocol {
    Http10,
    Http11,
    Http2,
    WebSocket,
    HttpsConnect,
}

#[allow(dead_code)]
impl ProxyProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyProtocol::Http10 => "HTTP/1.0",
            ProxyProtocol::Http11 => "HTTP/1.1",
            ProxyProtocol::Http2 => "HTTP/2",
            ProxyProtocol::WebSocket => "WebSocket",
            ProxyProtocol::HttpsConnect => "HTTPS CONNECT",
        }
    }
}

/// 测试代理配置
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestProxyConfig {
    pub id: String,
    pub port: u16,
    pub protocol: ProxyProtocol,
    pub auth_required: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub max_connections: usize,
}

impl TestProxyConfig {
    /// 创建新的测试代理配置（无认证）
    pub fn new(id: String, port: u16, protocol: ProxyProtocol) -> Self {
        Self {
            id,
            port,
            protocol,
            auth_required: false,
            username: None,
            password: None,
            max_connections: 100,
        }
    }

    /// 添加认证配置
    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.auth_required = true;
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    /// 生成代理地址字符串
    pub fn address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// 生成认证头（如果启用认证）
    pub fn auth_header(&self) -> Option<String> {
        if self.auth_required {
            if let (Some(user), Some(pass)) = (&self.username, &self.password) {
                let credentials = format!("{}:{}", user, pass);
                let encoded = BASE64_STANDARD.encode(credentials);
                return Some(format!("Basic {}", encoded));
            }
        }
        None
    }
}
