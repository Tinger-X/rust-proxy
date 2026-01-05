use crate::common::{CConfig, CProxy};
use reqwest::Client;

/// 测试 HTTPS CONNECT 带认证代理
#[tokio::test]
async fn test_https_auth() {
    let config = CConfig::TestProxyConfig::new(
        "https_auth".to_string(),
        18010,
        CConfig::ProxyProtocol::HttpsConnect,
    )
    .with_auth("testuser".to_string(), "testpass".to_string());

    let proxy = CProxy::TestProxy::start(config).await;

    // 创建 HTTP 客户端，使用带认证的代理
    let proxy_url = format!("http://testuser:testpass@127.0.0.1:{}", proxy.port());
    let client = Client::builder()
        .proxy(reqwest::Proxy::all(&proxy_url).unwrap())
        .build()
        .unwrap();

    // 发送 HTTPS 请求
    let response = client
        .get("https://httpbin.org/ip")
        .send()
        .await
        .expect("Failed to send request");

    // 验证响应
    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read response");
    assert!(body.contains("origin"));

    proxy.stop().await;
}
