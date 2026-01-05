use crate::std::config::ProxyConfig;
use reqwest::Client;
use serde_json::Value;

/// 测试 HTTP/2 GET 请求通过代理
#[tokio::test]
async fn test_http2_get_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 发送 HTTP/2 GET 请求
    let response = client
        .get("https://httpbin.org/get")
        .send()
        .await
        .expect("发送 HTTP/2 GET 请求失败");

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
    assert_eq!(
        json["url"], "https://httpbin.org/get",
        "响应中的 URL 不正确"
    );

    println!("HTTP/2 GET 请求测试通过");
}

/// 测试 HTTP/2 POST 请求通过代理
#[tokio::test]
async fn test_http2_post_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 构建请求体
    let request_body = serde_json::json!({
        "test": "http2_data",
        "protocol": "HTTP/2"
    });

    // 发送 HTTP/2 POST 请求
    let response = client
        .post("https://httpbin.org/post")
        .json(&request_body)
        .send()
        .await
        .expect("发送 HTTP/2 POST 请求失败");

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
        json["json"]["test"], "http2_data",
        "HTTP/2 请求数据未被正确转发"
    );

    println!("HTTP/2 POST 请求测试通过");
}

/// 测试 HTTP/2 多路复用（并发请求）
#[tokio::test]
async fn test_http2_multiplexing_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 发送多个并发请求
    let mut tasks = Vec::new();

    for i in 0..5 {
        let client_clone = client.clone();
        let task = tokio::spawn(async move {
            let response = client_clone
                .get(format!("https://httpbin.org/get?req={}", i))
                .send()
                .await
                .expect(&format!("HTTP/2 并发请求 {} 失败", i));

            assert_eq!(
                response.status().as_u16(),
                200,
                "HTTP/2 并发请求 {} 失败",
                i
            );

            let body = response.text().await.expect("读取响应体失败");
            let json: Value = serde_json::from_str(&body).expect("解析 JSON 失败");

            json["args"]["req"].as_i64().unwrap()
        });
        tasks.push(task);
    }

    // 等待所有请求完成
    let results = futures::future::join_all(tasks).await;

    // 验证所有请求都成功
    for (i, result) in results.into_iter().enumerate() {
        let value = result.expect("任务执行失败");
        assert_eq!(value, i as i64, "请求 {} 的响应值不匹配", i);
    }

    println!("HTTP/2 多路复用测试通过");
}

/// 测试 HTTP/2 自定义请求头
#[tokio::test]
async fn test_http2_custom_headers_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 发送带自定义头的 HTTP/2 请求
    let response = client
        .get("https://httpbin.org/headers")
        .header("X-Custom-Header", "HTTP2-Value")
        .header("X-Protocol-Header", "HTTP/2")
        .send()
        .await
        .expect("发送带自定义头的 HTTP/2 请求失败");

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
        json["headers"]["X-Custom-Header"], "HTTP2-Value",
        "自定义头 X-Custom-Header 未被正确转发"
    );
    assert_eq!(
        json["headers"]["X-Protocol-Header"], "HTTP/2",
        "自定义头 X-Protocol-Header 未被正确转发"
    );

    println!("HTTP/2 自定义请求头测试通过");
}

/// 测试 HTTP/2 持久连接（多流复用）
#[tokio::test]
async fn test_http2_stream_multiplexing_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 在同一个连接上发送多个请求
    for i in 0..10 {
        let response = client
            .get(format!("https://httpbin.org/get?stream={}", i))
            .send()
            .await
            .expect(&format!("HTTP/2 流请求 {} 失败", i));

        assert_eq!(
            response.status().as_u16(),
            200,
            "HTTP/2 流请求 {} 失败，状态码: {}",
            i,
            response.status()
        );
    }

    println!("HTTP/2 流多路复用测试通过");
}

/// 测试 HTTP/2 PUT 请求通过代理
#[tokio::test]
async fn test_http2_put_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 构建请求体
    let request_body = serde_json::json!({
        "action": "http2_update",
        "data": "http2_value"
    });

    // 发送 HTTP/2 PUT 请求
    let response = client
        .put("https://httpbin.org/put")
        .json(&request_body)
        .send()
        .await
        .expect("发送 HTTP/2 PUT 请求失败");

    // 验证状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "期望状态码 200，实际收到: {}",
        response.status()
    );

    println!("HTTP/2 PUT 请求测试通过");
}

/// 测试 HTTP/2 DELETE 请求通过代理
#[tokio::test]
async fn test_http2_delete_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 发送 HTTP/2 DELETE 请求
    let response = client
        .delete("https://httpbin.org/delete")
        .send()
        .await
        .expect("发送 HTTP/2 DELETE 请求失败");

    // 验证状态码
    assert_eq!(
        response.status().as_u16(),
        200,
        "期望状态码 200，实际收到: {}",
        response.status()
    );

    println!("HTTP/2 DELETE 请求测试通过");
}

/// 测试 HTTP/2 错误处理
#[tokio::test]
async fn test_http2_error_handling_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy = config.to_proxy().expect("无法创建代理客户端");

    // 创建 HTTP 客户端（强制使用 HTTP/2）
    let client = Client::builder()
        .proxy(proxy)
        .http2_prior_knowledge()
        .build()
        .expect("无法创建 HTTP/2 客户端");

    // 请求不存在的路径
    let response = client
        .get("https://httpbin.org/status/404")
        .send()
        .await
        .expect("发送 HTTP/2 请求失败");

    // 验证返回 404 状态码
    assert_eq!(
        response.status().as_u16(),
        404,
        "HTTP/2 应该返回 404 状态码"
    );

    println!("HTTP/2 错误处理测试通过");
}
