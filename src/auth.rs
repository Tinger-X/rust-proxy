use base64::Engine;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
}

impl AuthConfig {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn validate_proxy_auth(&self, auth_header: Option<&str>) -> bool {
        match auth_header {
            Some(header) => {
                if !header.starts_with("Basic ") {
                    warn!("不支持的认证类型: {}", header);
                    return false;
                }

                let encoded = &header[6..];
                match base64::engine::general_purpose::STANDARD.decode(encoded) {
                    Ok(decoded) => match String::from_utf8(decoded) {
                        Ok(credentials) => {
                            if let Some((username, password)) = credentials.split_once(':') {
                                let is_valid =
                                    username == self.username && password == self.password;
                                if is_valid {
                                    debug!("认证成功: {}", username);
                                } else {
                                    warn!("认证失败: {}", username);
                                }
                                is_valid
                            } else {
                                warn!("无效的认证凭据格式");
                                false
                            }
                        }
                        Err(e) => {
                            warn!("认证凭据不是有效的UTF-8: {}", e);
                            false
                        }
                    },
                    Err(e) => {
                        warn!("Base64解码失败: {}", e);
                        false
                    }
                }
            }
            None => false,
        }
    }

    pub fn generate_auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        format!("Basic {}", encoded)
    }
}

pub fn check_authentication(auth_config: &Option<AuthConfig>, auth_header: Option<&str>) -> bool {
    match auth_config {
        Some(config) => config.validate_proxy_auth(auth_header),
        None => true, // 没有配置认证则允许所有请求
    }
}
