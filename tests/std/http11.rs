use crate::std::config::ProxyConfig;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// 测试 HTTP/1.1 GET 请求通过代理
#[tokio::test]
async fn test_http11_get_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送 GET 请求
    let response = client
        .get("http://httpbin.org/get")
        .send()
        .await
        .expect("发送 GET 请求失败");

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

    // 验证 URL
    assert_eq!(json["url"], "http://httpbin.org/get", "响应中的 URL 不正确");

    println!("HTTP/1.1 GET 请求测试通过");
}

/// 测试 HTTP/1.1 POST 请求通过代理
#[tokio::test]
async fn test_http11_post_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 构建请求体
    let request_body = serde_json::json!({
        "test": "data",
        "message": "Hello, HTTP/1.1!"
    });

    // 发送 POST 请求
    let response = client
        .post("http://httpbin.org/post")
        .json(&request_body)
        .send()
        .await
        .expect("发送 POST 请求失败");

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
    assert_eq!(json["json"]["test"], "data", "请求数据未被正确转发");
    assert_eq!(
        json["json"]["message"], "Hello, HTTP/1.1!",
        "请求数据未被正确转发"
    );

    println!("HTTP/1.1 POST 请求测试通过");
}

/// 测试 HTTP/1.1 PUT 请求通过代理
#[tokio::test]
async fn test_http11_put_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 构建请求体
    let request_body = serde_json::json!({
        "action": "update",
        "data": "new_value"
    });

    // 发送 PUT 请求
    let response = client
        .put("http://httpbin.org/put")
        .json(&request_body)
        .send()
        .await
        .expect("发送 PUT 请求失败");

    // 验证状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "期望状态码 200，实际收到: {}",
        response.status()
    );

    println!("HTTP/1.1 PUT 请求测试通过");
}

/// 测试 HTTP/1.1 DELETE 请求通过代理
#[tokio::test]
async fn test_http11_delete_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送 DELETE 请求
    let response = client
        .delete("http://httpbin.org/delete")
        .send()
        .await
        .expect("发送 DELETE 请求失败");

    // 验证状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "期望状态码 200，实际收到: {}",
        response.status()
    );

    println!("HTTP/1.1 DELETE 请求测试通过");
}

/// 测试 HTTP/1.1 自定义请求头
#[tokio::test]
async fn test_http11_custom_headers_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送带自定义头的请求
    let response = client
        .get("http://httpbin.org/headers")
        .header("X-Custom-Header", "CustomValue")
        .header("X-Test-Header", "TestValue")
        .send()
        .await
        .expect("发送带自定义头的请求失败");

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
        json["headers"]["X-Custom-Header"], "CustomValue",
        "自定义头 X-Custom-Header 未被正确转发"
    );
    assert_eq!(
        json["headers"]["X-Test-Header"], "TestValue",
        "自定义头 X-Test-Header 未被正确转发"
    );

    println!("HTTP/1.1 自定义请求头测试通过");
}

/// 测试 HTTP/1.1 持久连接（Keep-Alive）
#[tokio::test]
async fn test_http11_keep_alive_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（启用连接池）
    let client = Client::builder()
        .proxy(proxy)
        .pool_max_idle_per_host(5)
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送多个请求（复用连接）
    for i in 0..3 {
        let response = client
            .get(format!("http://httpbin.org/get?req={}", i))
            .send()
            .await
            .expect(&format!("发送第 {} 个请求失败", i + 1));

        assert_eq!(
            response.status().as_u16(),
            200,
            "第 {} 个请求失败，状态码: {}",
            i + 1,
            response.status()
        );
    }

    println!("HTTP/1.1 持久连接测试通过");
}

/// 测试 HTTP/1.1 响应头
#[tokio::test]
async fn test_http11_response_headers_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端
    let client = Client::builder()
        .proxy(proxy)
        .build()
        .expect("无法创建 HTTP 客户端");

    // 发送请求
    let response = client
        .get("http://httpbin.org/get")
        .send()
        .await
        .expect("发送请求失败");

    // 验证响应头
    assert!(
        response.headers().contains_key("content-type"),
        "响应缺少 Content-Type 头"
    );

    let content_type = response.headers().get("content-type").unwrap();
    let content_type_str = content_type
        .to_str()
        .expect("Content-Type 不是有效的 UTF-8");
    assert!(
        content_type_str.contains("application/json"),
        "Content-Type 应该是 application/json"
    );

    println!("HTTP/1.1 响应头测试通过");
}
