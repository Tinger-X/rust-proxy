/// 协议类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolType {
    /// HTTP/1.0
    Http10,
    /// HTTP/1.1
    Http11,
    /// HTTP/2 (clear-text)
    Http2,
    /// WebSocket Upgrade
    WebSocketUpgrade {
        key: String,
        host: String,
        port: u16,
        version: String,
    },
    /// CONNECT 隧道 (HTTPS)
    ConnectTunnel { 
        host: String, 
        port: u16, 
        version: String 
    },
    /// 未知协议
    Unknown,
}

/// 检测协议类型
///
/// 根据初始字节流判断客户端使用的协议类型
pub fn detect_protocol(buffer: &[u8]) -> ProtocolType {
    // 检查HTTP/2 preface
    if is_http2_preface(buffer) {
        return ProtocolType::Http2;
    }

    // 尝试解析为HTTP/1.x请求
    if let Some(method) = parse_http_method(buffer) {
        // 检查是否是CONNECT请求
        if method == "CONNECT" {
            if let Some((host, port)) = parse_connect_target(buffer) {
                let version = parse_http_version(buffer).unwrap_or_else(|| "HTTP/1.1".to_string());
                return ProtocolType::ConnectTunnel { 
                    host, 
                    port, 
                    version 
                };
            }
        }

        // 检查是否是WebSocket升级请求
        if is_websocket_upgrade(buffer) {
            if let Some((host, port, key)) = parse_websocket_details(buffer) {
                let version = parse_http_version(buffer).unwrap_or_else(|| "HTTP/1.1".to_string());
                return ProtocolType::WebSocketUpgrade { 
                    key, 
                    host, 
                    port, 
                    version 
                };
            }
        }

        // 检查HTTP版本
        if let Some(version) = parse_http_version(buffer) {
            match version.as_str() {
                "HTTP/1.0" => return ProtocolType::Http10,
                "HTTP/1.1" => return ProtocolType::Http11,
                _ => {}
            }
        }

        // 默认认为是HTTP/1.1
        return ProtocolType::Http11;
    }

    ProtocolType::Unknown
}

/// 检查是否是HTTP/2 preface
/// HTTP/2 preface: "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
fn is_http2_preface(buffer: &[u8]) -> bool {
    const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

    if buffer.len() < HTTP2_PREFACE.len() {
        return false;
    }

    &buffer[..HTTP2_PREFACE.len()] == HTTP2_PREFACE
}

/// 解析HTTP方法
fn parse_http_method(buffer: &[u8]) -> Option<String> {
    let request = String::from_utf8_lossy(buffer);
    let first_line = request.lines().next()?;

    let method = first_line.split_whitespace().next()?;
    Some(method.to_uppercase())
}

/// 解析HTTP版本
fn parse_http_version(buffer: &[u8]) -> Option<String> {
    let request = String::from_utf8_lossy(buffer);
    let first_line = request.lines().next()?;

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() >= 3 {
        Some(parts[2].to_uppercase())
    } else {
        None
    }
}

/// 解析CONNECT请求的目标
fn parse_connect_target(buffer: &[u8]) -> Option<(String, u16)> {
    let request = String::from_utf8_lossy(buffer);
    let first_line = request.lines().next()?;

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

/// 检查是否是WebSocket升级请求
fn is_websocket_upgrade(buffer: &[u8]) -> bool {
    let request = String::from_utf8_lossy(buffer).to_lowercase();

    request.contains("upgrade: websocket") && request.contains("connection: upgrade")
}

/// 解析WebSocket握手详细信息
fn parse_websocket_details(buffer: &[u8]) -> Option<(String, u16, String)> {
    let request = String::from_utf8_lossy(buffer);

    // 提取Sec-WebSocket-Key
    let ws_key = request
        .lines()
        .find(|line| line.to_lowercase().starts_with("sec-websocket-key:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim().to_string())?;

    // 提取Host头
    let host_line = request
        .lines()
        .find(|line| line.to_lowercase().starts_with("host:"))?;

    let host_value = host_line.split(':').nth(1)?.trim();

    // 解析host:port
    let mut parts = host_value.split(':');
    let host = parts.next()?.to_string();
    let port = if let Some(port_str) = parts.next() {
        port_str.parse::<u16>().ok()?
    } else {
        // WebSocket默认端口: ws=80, wss=443
        // 这里简化处理，默认80
        80
    };

    Some((host, port, ws_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http2_preface_detection() {
        let buffer = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
        assert_eq!(detect_protocol(buffer), ProtocolType::Http2);
    }

    #[test]
    fn test_http10_detection() {
        let buffer = b"GET / HTTP/1.0\r\nHost: example.com\r\n\r\n";
        assert_eq!(detect_protocol(buffer), ProtocolType::Http10);
    }

    #[test]
    fn test_http11_detection() {
        let buffer = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert_eq!(detect_protocol(buffer), ProtocolType::Http11);
    }

    #[test]
    fn test_connect_detection() {
        let buffer = b"CONNECT example.com:443 HTTP/1.1\r\n\r\n";
        assert_eq!(
            detect_protocol(buffer),
            ProtocolType::ConnectTunnel {
                host: "example.com".to_string(),
                port: 443,
                version: "HTTP/1.1".to_string()
            }
        );
    }

    #[test]
    fn test_websocket_upgrade_detection() {
        let buffer = b"GET /chat HTTP/1.1\r\n\
            Host: example.com\r\n\
            Upgrade: websocket\r\n\
            Connection: Upgrade\r\n\
            Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";

        match detect_protocol(buffer) {
            ProtocolType::WebSocketUpgrade { key, host, port, version } => {
                assert_eq!(key, "dGhlIHNhbXBsZSBub25jZQ==");
                assert_eq!(host, "example.com");
                assert_eq!(port, 80);
                assert_eq!(version, "HTTP/1.1".to_string());
            }
            _ => panic!("Expected WebSocket upgrade"),
        }
    }
}
