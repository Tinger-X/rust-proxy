use crate::common::{CConfig, CProxy};
use reqwest::Client;

/// 测试 HTTPS CONNECT 无认证代理
#[tokio::test]
async fn test_https_no_auth() {
    let config = CConfig::TestProxyConfig::new(
        "https_noauth".to_string(),
        18005,
        CConfig::ProxyProtocol::HttpsConnect,
    );

    let proxy = CProxy::TestProxy::start(config).await;

    // 创建 HTTP 客户端，使用代理
    let proxy_url = format!("http://127.0.0.1:{}", proxy.port());
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
