use crate::std::config::ProxyConfig;
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::WebSocketStream;

/// 测试 WebSocket 连接通过代理（无认证）
#[tokio::test]
async fn test_websocket_via_proxy_no_auth() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");

    // 如果需要认证，跳过此测试
    if config.requires_auth() {
        println!("代理需要认证，跳过无认证 WebSocket 测试");
        return;
    }

    // 构建 WebSocket 连接 URL
    let ws_url = "wss://echo.websocket.org";

    // 构建带有代理的请求
    let request = Request::builder()
        .uri(ws_url)
        .header("Host", "echo.websocket.org")
        .body(())
        .expect("无法构建 WebSocket 请求");

    // 连接 WebSocket（通过代理）
    let (mut ws_stream, _) = tokio_tungstenite::connect_async_with_config(request, None, false)
        .await
        .expect("无法通过代理连接到 WebSocket 服务器");

    // 发送测试消息
    let test_message = "Hello, WebSocket via proxy!";
    ws_stream
        .send(Message::Text(test_message.to_string()))
        .await
        .expect("发送 WebSocket 消息失败");

    // 接收回显消息
    let response = ws_stream
        .next()
        .await
        .expect("未收到响应")
        .expect("接收消息失败");

    // 验证回显消息
    match response {
        Message::Text(text) => {
            assert_eq!(text, test_message, "回显消息不匹配");
            println!("WebSocket 回显消息验证成功: {}", text);
        }
        _ => panic!("期望收到文本消息"),
    }

    // 关闭连接
    ws_stream
        .send(Message::Close(None))
        .await
        .expect("关闭 WebSocket 连接失败");

    println!("WebSocket 连接测试（无认证）通过");
}

/// 测试 WebSocket 连接通过代理（带认证）
#[tokio::test]
async fn test_websocket_via_proxy_with_auth() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");

    // 如果不需要认证，跳过此测试
    if !config.requires_auth() {
        println!("代理不需要认证，跳过认证 WebSocket 测试");
        return;
    }

    // 构建 WebSocket 连接 URL
    let ws_url = "wss://echo.websocket.org";

    let auth_header = config
        .auth_header()
        .expect("应该有认证头");

    // 构建带有代理认证的请求
    let request = Request::builder()
        .uri(ws_url)
        .header("Host", "echo.websocket.org")
        .header("Proxy-Authorization", auth_header)
        .body(())
        .expect("无法构建 WebSocket 请求");

    // 连接 WebSocket（通过代理）
    let (mut ws_stream, _) = tokio_tungstenite::connect_async_with_config(request, None, false)
        .await
        .expect("无法通过代理连接到 WebSocket 服务器");

    // 发送测试消息
    let test_message = "Hello, WebSocket with auth!";
    ws_stream
        .send(Message::Text(test_message.to_string()))
        .await
        .expect("发送 WebSocket 消息失败");

    // 接收回显消息
    let response = ws_stream
        .next()
        .await
        .expect("未收到响应")
        .expect("接收消息失败");

    // 验证回显消息
    match response {
        Message::Text(text) => {
            assert_eq!(text, test_message, "回显消息不匹配");
            println!("WebSocket 回显消息验证成功（带认证）: {}", text);
        }
        _ => panic!("期望收到文本消息"),
    }

    // 关闭连接
    ws_stream
        .send(Message::Close(None))
        .await
        .expect("关闭 WebSocket 连接失败");

    println!("WebSocket 连接测试（带认证）通过");
}

/// 测试 WebSocket 多条消息收发
#[tokio::test]
async fn test_websocket_multiple_messages_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");

    // 构建 WebSocket 连接 URL
    let ws_url = "wss://echo.websocket.org";

    // 构建请求
    let request = if config.requires_auth() {
        let auth_header = config.auth_header().expect("应该有认证头");
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .header("Proxy-Authorization", auth_header)
            .body(())
            .expect("无法构建 WebSocket 请求")
    } else {
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .body(())
            .expect("无法构建 WebSocket 请求")
    };

    // 连接 WebSocket
    let mut ws_stream = connect_websocket(request).await;

    // 发送多条消息
    let messages = vec![
        "Message 1",
        "Message 2",
        "Message 3",
        "Message 4",
        "Message 5",
    ];

    for msg in messages.iter() {
        ws_stream
            .send(Message::Text(msg.to_string()))
            .await
            .expect("发送 WebSocket 消息失败");
    }

    // 接收并验证所有回显消息
    for msg in messages.iter() {
        let response = ws_stream
            .next()
            .await
            .expect("未收到响应")
            .expect("接收消息失败");

        match response {
            Message::Text(text) => {
                assert_eq!(text, *msg, "回显消息不匹配");
            }
            _ => panic!("期望收到文本消息"),
        }
    }

    // 关闭连接
    ws_stream
        .send(Message::Close(None))
        .await
        .expect("关闭 WebSocket 连接失败");

    println!("WebSocket 多条消息测试通过");
}

/// 测试 WebSocket Ping/Pong
#[tokio::test]
async fn test_websocket_ping_pong_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");

    // 构建 WebSocket 连接 URL
    let ws_url = "wss://echo.websocket.org";

    // 构建请求
    let request = if config.requires_auth() {
        let auth_header = config.auth_header().expect("应该有认证头");
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .header("Proxy-Authorization", auth_header)
            .body(())
            .expect("无法构建 WebSocket 请求")
    } else {
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .body(())
            .expect("无法构建 WebSocket 请求")
    };

    // 连接 WebSocket
    let mut ws_stream = connect_websocket(request).await;

    // 发送 Ping 消息
    ws_stream
        .send(Message::Ping(vec![1, 2, 3, 4]))
        .await
        .expect("发送 Ping 消息失败");

    // 等待 Pong 响应
    tokio::time::timeout(Duration::from_secs(5), async {
        match ws_stream.next().await {
            Some(Ok(Message::Pong(data))) => {
                assert_eq!(data, vec![1, 2, 3, 4], "Pong 数据不匹配");
            }
            _ => panic!("期望收到 Pong 消息"),
        }
    })
    .await
    .expect("等待 Pong 响应超时");

    // 关闭连接
    ws_stream
        .send(Message::Close(None))
        .await
        .expect("关闭 WebSocket 连接失败");

    println!("WebSocket Ping/Pong 测试通过");
}

/// 测试 WebSocket 二进制消息
#[tokio::test]
async fn test_websocket_binary_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");

    // 构建 WebSocket 连接 URL
    let ws_url = "wss://echo.websocket.org";

    // 构建请求
    let request = if config.requires_auth() {
        let auth_header = config.auth_header().expect("应该有认证头");
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .header("Proxy-Authorization", auth_header)
            .body(())
            .expect("无法构建 WebSocket 请求")
    } else {
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .body(())
            .expect("无法构建 WebSocket 请求")
    };

    // 连接 WebSocket
    let mut ws_stream = connect_websocket(request).await;

    // 发送二进制消息
    let binary_data: Vec<u8> = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in bytes
    ws_stream
        .send(Message::Binary(binary_data.clone()))
        .await
        .expect("发送二进制消息失败");

    // 接收回显消息
    let response = ws_stream
        .next()
        .await
        .expect("未收到响应")
        .expect("接收消息失败");

    // 验证回显消息
    match response {
        Message::Binary(data) => {
            assert_eq!(data, binary_data, "回显二进制数据不匹配");
            let text = String::from_utf8(data).unwrap();
            println!("WebSocket 二进制消息验证成功: {}", text);
        }
        _ => panic!("期望收到二进制消息"),
    }

    // 关闭连接
    ws_stream
        .send(Message::Close(None))
        .await
        .expect("关闭 WebSocket 连接失败");

    println!("WebSocket 二进制消息测试通过");
}

/// 测试 WebSocket 连接稳定性
#[tokio::test]
async fn test_websocket_stability_via_proxy() {
    let config = ProxyConfig::from_env().expect("无法加载代理配置");

    // 构建 WebSocket 连接 URL
    let ws_url = "wss://echo.websocket.org";

    // 构建请求
    let request = if config.requires_auth() {
        let auth_header = config.auth_header().expect("应该有认证头");
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .header("Proxy-Authorization", auth_header)
            .body(())
            .expect("无法构建 WebSocket 请求")
    } else {
        Request::builder()
            .uri(ws_url)
            .header("Host", "echo.websocket.org")
            .body(())
            .expect("无法构建 WebSocket 请求")
    };

    // 连接 WebSocket
    let mut ws_stream = connect_websocket(request).await;

    // 持续发送和接收消息
    for i in 0..10 {
        let test_message = format!("Stability test message {}", i);
        ws_stream
            .send(Message::Text(test_message.clone()))
            .await
            .expect("发送消息失败");

        let response = ws_stream
            .next()
            .await
            .expect("未收到响应")
            .expect("接收消息失败");

        match response {
            Message::Text(text) => {
                assert_eq!(text, test_message, "消息 {} 不匹配", i);
            }
            _ => panic!("期望收到文本消息"),
        }
    }

    // 关闭连接
    ws_stream
        .send(Message::Close(None))
        .await
        .expect("关闭 WebSocket 连接失败");

    println!("WebSocket 连接稳定性测试通过");
}

/// 辅助函数：连接 WebSocket（通过代理）
async fn connect_websocket(request: Request) -> WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let (ws_stream, _) = tokio_tungstenite::connect_async_with_config(request, None, false)
        .await
        .expect("无法连接到 WebSocket 服务器");
    ws_stream
}
