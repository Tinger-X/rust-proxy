use crate::std::config::ProxyConfig;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// 测试 HTTP/1.0 GET 请求通过代理
#[tokio::test]
async fn test_http10_get_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy_addr = config.address();

    // 连接到代理服务器
    let mut stream =
        TcpStream::connect(&proxy_addr).expect(&format!("无法连接到代理服务器: {}", proxy_addr));
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("设置读取超时失败");

    // 构建 HTTP/1.0 请求
    let request = format!(
        "GET http://httpbin.org/get HTTP/1.0\r\n\
         Host: httpbin.org\r\n\
         User-Agent: RustProxy-Test/1.0\r\n\
         Connection: close\r\n\r\n"
    );

    // 发送请求
    stream
        .write_all(request.as_bytes())
        .expect("发送 HTTP 请求失败");

    // 读取响应
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    // 读取状态行
    let status_line = lines.next().unwrap().expect("无法读取状态行");
    println!("HTTP/1.0 状态行: {}", status_line);

    // 验证状态码
    assert!(
        status_line.contains("200"),
        "期望状态码 200，实际收到: {}",
        status_line
    );

    // 读取响应头
    let mut has_content_length = false;
    let mut has_content_type = false;

    loop {
        let line = match lines.next() {
            Some(Ok(l)) if l.is_empty() => break,
            Some(Ok(l)) => l,
            Some(Err(e)) => panic!("读取响应头失败: {}", e),
            None => break,
        };

        if line.contains("Content-Length") {
            has_content_length = true;
        }
        if line.contains("Content-Type") {
            has_content_type = true;
        }
    }

    // 验证响应头
    assert!(has_content_length, "响应缺少 Content-Length 头");
    assert!(has_content_type, "响应缺少 Content-Type 头");

    println!("HTTP/1.0 GET 请求测试通过");
}

/// 测试 HTTP/1.0 POST 请求通过代理
#[tokio::test]
async fn test_http10_post_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy_addr = config.address();

    // 连接到代理服务器
    let mut stream =
        TcpStream::connect(&proxy_addr).expect(&format!("无法连接到代理服务器: {}", proxy_addr));
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("设置读取超时失败");

    // 构建请求体
    let body = r#"{"test": "data"}"#;
    let content_length = body.len();

    // 构建 HTTP/1.0 POST 请求
    let request = format!(
        "POST http://httpbin.org/post HTTP/1.0\r\n\
         Host: httpbin.org\r\n\
         User-Agent: RustProxy-Test/1.0\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n{}",
        content_length, body
    );

    // 发送请求
    stream
        .write_all(request.as_bytes())
        .expect("发送 HTTP POST 请求失败");

    // 读取响应
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    // 读取状态行
    let status_line = lines.next().unwrap().expect("无法读取状态行");
    println!("HTTP/1.0 POST 状态行: {}", status_line);

    // 验证状态码
    assert!(
        status_line.contains("200"),
        "期望状态码 200，实际收到: {}",
        status_line
    );

    println!("HTTP/1.0 POST 请求测试通过");
}

/// 测试 HTTP/1.0 HEAD 请求通过代理
#[tokio::test]
async fn test_http10_head_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy_addr = config.address();

    // 连接到代理服务器
    let mut stream =
        TcpStream::connect(&proxy_addr).expect(&format!("无法连接到代理服务器: {}", proxy_addr));
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("设置读取超时失败");

    // 构建 HTTP/1.0 HEAD 请求
    let request = format!(
        "HEAD http://httpbin.org/get HTTP/1.0\r\n\
         Host: httpbin.org\r\n\
         User-Agent: RustProxy-Test/1.0\r\n\
         Connection: close\r\n\r\n"
    );

    // 发送请求
    stream
        .write_all(request.as_bytes())
        .expect("发送 HTTP HEAD 请求失败");

    // 读取响应
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    // 读取状态行
    let status_line = lines.next().unwrap().expect("无法读取状态行");
    println!("HTTP/1.0 HEAD 状态行: {}", status_line);

    // 验证状态码
    assert!(
        status_line.contains("200"),
        "期望状态码 200，实际收到: {}",
        status_line
    );

    // HEAD 请求不应该有响应体
    let mut has_body = false;
    for line in lines.flatten() {
        if !line.is_empty() {
            has_body = true;
            break;
        }
    }

    assert!(!has_body, "HEAD 请求不应该有响应体");

    println!("HTTP/1.0 HEAD 请求测试通过");
}

/// 测试 HTTP/1.0 自定义请求头
#[tokio::test]
async fn test_http10_custom_headers_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");
    let proxy_addr = config.address();

    // 连接到代理服务器
    let mut stream =
        TcpStream::connect(&proxy_addr).expect(&format!("无法连接到代理服务器: {}", proxy_addr));
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("设置读取超时失败");

    // 构建 HTTP/1.0 请求（带自定义头）
    let request = format!(
        "GET http://httpbin.org/headers HTTP/1.0\r\n\
         Host: httpbin.org\r\n\
         User-Agent: RustProxy-Test/1.0\r\n\
         X-Custom-Header: CustomValue\r\n\
         X-Test-Header: TestValue\r\n\
         Connection: close\r\n\r\n"
    );

    // 发送请求
    stream
        .write_all(request.as_bytes())
        .expect("发送 HTTP 请求失败");

    // 读取响应
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    // 读取状态行
    let status_line = lines.next().unwrap().expect("无法读取状态行");
    println!("HTTP/1.0 自定义头状态行: {}", status_line);

    // 验证状态码
    assert!(
        status_line.contains("200"),
        "期望状态码 200，实际收到: {}",
        status_line
    );

    // 读取响应体，查找自定义头
    let response_body: String = lines.filter_map(|l| l.ok()).collect();

    // 验证自定义头是否被代理转发
    assert!(
        response_body.contains("X-Custom-Header"),
        "自定义头 X-Custom-Header 未被代理转发"
    );
    assert!(
        response_body.contains("CustomValue"),
        "自定义头值 CustomValue 未找到"
    );

    println!("HTTP/1.0 自定义请求头测试通过");
}
