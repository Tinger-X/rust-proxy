use crate::std::config::ProxyConfig;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// 测试 HTTPS CONNECT GET 请求通过代理
#[tokio::test]
async fn test_https_get_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送 HTTPS GET 请求
    let response = client
        .get("https://httpbin.org/ip")
        .send()
        .await
        .expect("发送 HTTPS GET 请求失败");

    // 验证状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "期望状态码 200，实际收到: {}",
        response.status()
    );

    // 验证响应内容
    let body = response.text().await.expect("读取响应体失败");

    let json: Value = serde_json::from_str(&body).expect("解析 JSON 失败");

    // 验证响应包含 origin 字段
    assert!(json.get("origin").is_some(), "响应缺少 origin 字段");

    println!("HTTPS CONNECT GET 请求测试通过");
}

/// 测试 HTTPS CONNECT POST 请求通过代理
#[tokio::test]
async fn test_https_post_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 构建请求体
    let request_body = serde_json::json!({
        "test": "https_data",
        "secure": true
    });

    // 发送 HTTPS POST 请求
    let response = client
        .post("https://httpbin.org/post")
        .json(&request_body)
        .send()
        .await
        .expect("发送 HTTPS POST 请求失败");

    // 验证状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "期望状态码 200，实际收到: {}",
        response.status()
    );

    // 验证响应内容
    let body = response.text().await.expect("读取响应体失败");

    let json: Value = serde_json::from_str(&body).expect("解析 JSON 失败");

    // 验证请求数据被正确转发
    assert_eq!(
        json["json"]["test"], "https_data",
        "HTTPS 请求数据未被正确转发"
    );

    println!("HTTPS CONNECT POST 请求测试通过");
}

/// 测试 HTTPS 自定义请求头
#[tokio::test]
async fn test_https_custom_headers_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送带自定义头的 HTTPS 请求
    let response = client
        .get("https://httpbin.org/headers")
        .header("X-Custom-Header", "CustomHTTPS")
        .header("X-Security-Header", "SecureValue")
        .send()
        .await
        .expect("发送带自定义头的 HTTPS 请求失败");

    // 验证状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "期望状态码 200，实际收到: {}",
        response.status()
    );

    // 验证响应内容
    let body = response.text().await.expect("读取响应体失败");

    let json: Value = serde_json::from_str(&body).expect("解析 JSON 失败");

    // 验证自定义头被正确转发
    assert_eq!(
        json["headers"]["X-Custom-Header"], "CustomHTTPS",
        "自定义头 X-Custom-Header 未被正确转发"
    );
    assert_eq!(
        json["headers"]["X-Security-Header"], "SecureValue",
        "自定义头 X-Security-Header 未被正确转发"
    );

    println!("HTTPS 自定义请求头测试通过");
}

/// 测试 HTTPS 响应完整性
#[tokio::test]
async fn test_https_response_integrity_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送 HTTPS 请求
    let response = client
        .get("https://httpbin.org/get")
        .send()
        .await
        .expect("发送 HTTPS 请求失败");

    // 验证响应头
    assert!(
        response.headers().contains_key("content-type"),
        "HTTPS 响应缺少 Content-Type 头"
    );

    // 验证 HTTPS 状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "HTTPS 期望状态码 200，实际收到: {}",
        response.status()
    );

    // 读取并验证响应体
    let body = response.text().await.expect("读取 HTTPS 响应体失败");

    let json: Value = serde_json::from_str(&body).expect("解析 JSON 失败");

    // 验证响应 URL 包含 https
    assert!(
        json["url"].as_str().unwrap().starts_with("https://"),
        "响应 URL 应该使用 HTTPS"
    );

    println!("HTTPS 响应完整性测试通过");
}

/// 测试 HTTPS 持久连接（Keep-Alive）
#[tokio::test]
async fn test_https_keep_alive_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（启用连接池）
    let client = Client::builder()
        .proxy(proxy)
        .pool_max_idle_per_host(5)
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送多个 HTTPS 请求（复用连接）
    for i in 0..3 {
        let response = client
            .get(format!("https://httpbin.org/get?req={}", i))
            .send()
            .await
            .expect(&format!("发送第 {} 个 HTTPS 请求失败", i + 1));

        assert_eq!(
            response.status().as_u16(),
            200,
            "第 {} 个 HTTPS 请求失败，状态码: {}",
            i + 1,
            response.status()
        );
    }

    println!("HTTPS 持久连接测试通过");
}

/// 测试 HTTPS 错误处理
#[tokio::test]
async fn test_https_error_handling_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 请求不存在的路径
    let response = client
        .get("https://httpbin.org/status/404")
        .send()
        .await
        .expect("发送 HTTPS 请求失败");

    // 验证返回 404 状态码
    assert_eq!(response.status().as_u16(), 404, "HTTPS 应该返回 404 状态码");

    println!("HTTPS 错误处理测试通过");
}

/// 测试 HTTPS 内容类型
#[tokio::test]
async fn test_https_content_type_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 请求 JSON 内容
    let response = client
        .get("https://httpbin.org/get")
        .send()
        .await
        .expect("发送 HTTPS 请求失败");

    // 验证 Content-Type
    let content_type = response
        .headers()
        .get("content-type")
        .expect("HTTPS 响应缺少 Content-Type 头");
    let content_type_str = content_type
        .to_str()
        .expect("Content-Type 不是有效的 UTF-8");

    assert!(
        content_type_str.contains("application/json"),
        "HTTPS Content-Type 应该是 application/json"
    );

    println!("HTTPS 内容类型测试通过");
}
