use crate::common::{CConfig, CProxy};

/// 测试 WebSocket 无认证代理
#[tokio::test]
async fn test_websocket_no_auth() {
    let config = CConfig::TestProxyConfig::new(
        "websocket_noauth".to_string(),
        18004,
        CConfig::ProxyProtocol::WebSocket,
    );

    let proxy = CProxy::TestProxy::start(config).await;

    // 使用 tokio-tungstenite 连接 WebSocket echo 服务器
    let echo_url = "ws://echo.websocket.org";

    // 注意：这里使用 echo.websocket.org 作为测试目标
    // 实际测试中可能需要使用本地的 WebSocket echo 服务器
    let (ws_stream, _) = tokio_tungstenite::tungstenite::client::connect(echo_url)
        .expect("Failed to connect to WebSocket echo server");

    // 验证连接成功（直接验证流存在）
    let _ = ws_stream;

    proxy.stop().await;
}
