/// 标准代理测试模块
///
/// 此模块用于测试已运行的代理服务器对各种网络协议的支持情况。
/// 所有测试从 `.env` 文件读取代理配置（IP、端口、用户名、密码）。
///
/// 配置文件格式：
/// ```bash
/// PROXY_IP=127.0.0.1
/// PROXY_PORT=8080
/// PROXY_USERNAME=         # 可选，留空表示不使用认证
/// PROXY_PASSWORD=         # 可选，留空表示不使用认证
/// ```

pub mod config;

// HTTP/1.0 测试
pub mod http10;

// HTTP/1.1 测试
pub mod http11;

// HTTPS 测试
pub mod https;

// HTTP/2 测试
pub mod http2;

// WebSocket 测试
pub mod websocket;

// 公开配置模块以便其他模块使用
pub use config::ProxyConfig;
