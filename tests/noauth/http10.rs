use crate::common::{CConfig, CProxy};
use reqwest::Client;

/// 测试 HTTP/1.0 无认证代理
#[tokio::test]
async fn test_http10_no_auth() {
    let config = CConfig::TestProxyConfig::new(
        "http10_noauth".to_string(),
        18001,
        CConfig::ProxyProtocol::Http10,
    );

    let proxy = CProxy::TestProxy::start(config).await;

    // 创建 HTTP 客户端，使用代理
    let proxy_url = format!("http://127.0.0.1:{}", proxy.port());
    let client = Client::builder()
        .proxy(reqwest::Proxy::all(&proxy_url).unwrap())
        .build()
        .unwrap();

    // 发送 HTTP 请求到 httpbin.org
    let response = client
        .get("http://httpbin.org/get")
        .send()
        .await
        .expect("Failed to send request");

    // 验证响应状态码
    assert_eq!(response.status(), 200);

    // 验证响应内容
    let body = response.text().await.expect("Failed to read response");
    assert!(body.contains("\"url\""));
    assert!(body.contains("httpbin.org"));

    proxy.stop().await;
}
