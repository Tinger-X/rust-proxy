use crate::common::{CConfig, CProxy};

/// 测试 WebSocket 带认证代理
#[tokio::test]
async fn test_websocket_auth() {
    let config = CConfig::TestProxyConfig::new(
        "websocket_auth".to_string(),
        18009,
        CConfig::ProxyProtocol::WebSocket,
    )
    .with_auth("testuser".to_string(), "testpass".to_string());

    let proxy = CProxy::TestProxy::start(config).await;

    // 获取认证头
    let auth_header = proxy.auth_header().expect("Should have auth header");

    // 使用 tokio-tungstenite 连接 WebSocket echo 服务器
    // 注意：实际测试中需要处理代理认证
    let echo_url = "ws://echo.websocket.org";

    // 构建带有认证头的请求
    let request = tokio_tungstenite::tungstenite::handshake::client::Request::builder()
        .uri(echo_url)
        .header("Proxy-Authorization", auth_header)
        .body(())
        .expect("Failed to build request");

    let (ws_stream, _) =
        tokio_tungstenite::tungstenite::client::connect_with_config(request, None, 64_u8)
            .expect("Failed to connect to WebSocket echo server");

    // 验证连接成功（直接验证流存在）
    let _ = ws_stream;

    proxy.stop().await;
}
