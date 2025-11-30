use std::time::Duration;
use reqwest::{Proxy, Client};
use tokio::time::timeout;
use tracing::{info, error, warn};
use dotenv::from_filename;

#[derive(Debug, Clone)]
struct TestConfig {
    proxy_host: String,
    proxy_port: u16,
    target_url: String,
    test_count: usize,
    username: Option<String>,
    password: Option<String>,
}

impl TestConfig {
    fn from_env() -> Self {
        // 尝试加载tests/.env文件，如果失败也不影响测试
        if let Err(e) = from_filename("tests/.env") {
            warn!("无法加载tests/.env文件: {}，使用默认配置", e);
        } else {
            info!("✅ 已加载tests/.env文件配置");
        }
        
        Self {
            proxy_host: std::env::var("PROXY_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            proxy_port: std::env::var("PROXY_PORT")
                .unwrap_or_else(|_| "24975".to_string())
                .parse()
                .unwrap_or(24975),
            target_url: std::env::var("TARGET_URL")
                .unwrap_or_else(|_| "https://www.baidu.com".to_string()),
            test_count: std::env::var("TEST_COUNT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            username: std::env::var("PROXY_USERNAME").ok(),
            password: std::env::var("PROXY_PASSWORD").ok(),
        }
    }
    
    fn print_info(&self) {
        info!("📡 代理服务器: {}:{}", self.proxy_host, self.proxy_port);
        info!("🎯 目标URL: {}", self.target_url);
        info!("🔄 测试次数: {}", self.test_count);
        if let (Some(username), Some(_)) = (&self.username, &self.password) {
            info!("🔐 认证用户: {}", username);
        } else {
            info!("🔓 无认证模式");
        }
        
        // 显示配置来源
        if std::env::var("PROXY_HOST").is_ok() {
            info!("📄 配置来源: .env文件或环境变量");
        } else {
            info!("📄 配置来源: 默认值");
        }
    }
}

#[tokio::test]
async fn test_proxy_without_auth() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始代理服务器黑盒测试（无认证模式）");
    
    let config = TestConfig::from_env();
    config.print_info();

    // 创建HTTP客户端
    let proxy_url = format!("http://{}:{}", config.proxy_host, config.proxy_port);
    let proxy = Proxy::all(&proxy_url)?;
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(30))
        .user_agent("python-requests/2.31.0")  // 模拟Python requests的User-Agent
        .danger_accept_invalid_certs(true)     // 更宽松的证书验证
        .build()?;

    // 执行测试
    run_tests(&client, &config.target_url, config.test_count).await?;

    Ok(())
}

#[tokio::test]
async fn test_proxy_with_auth() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始代理服务器黑盒测试（认证模式）");
    
    let config = TestConfig::from_env();
    
    // 检查是否有认证信息
    if config.username.is_none() || config.password.is_none() {
        return Err("认证测试需要设置 PROXY_USERNAME 和 PROXY_PASSWORD 环境变量".into());
    }
    
    config.print_info();

    // 创建HTTP客户端
    let proxy_url = format!("http://{}:{}", config.proxy_host, config.proxy_port);
    let proxy = Proxy::all(&proxy_url)?
        .basic_auth(&config.username.unwrap(), &config.password.unwrap());
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(30))
        .user_agent("python-requests/2.31.0")  // 模拟Python requests的User-Agent
        .danger_accept_invalid_certs(true)     // 更宽松的证书验证
        .build()?;

    // 执行测试
    run_tests(&client, &config.target_url, config.test_count).await?;

    Ok(())
}

#[tokio::test]
async fn test_proxy_http_target() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始代理服务器HTTP目标测试");
    
    let config = TestConfig::from_env();
    config.print_info();

    // 创建HTTP客户端
    let proxy_url = format!("http://{}:{}", config.proxy_host, config.proxy_port);
    let proxy = Proxy::all(&proxy_url)?;
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(30))
        .user_agent("python-requests/2.31.0")  // 模拟Python requests的User-Agent
        .danger_accept_invalid_certs(true)     // 更宽松的证书验证
        .build()?;

    // 执行测试
    run_tests(&client, &config.target_url, config.test_count).await?;

    Ok(())
}

#[tokio::test]
async fn test_proxy_performance() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始代理服务器性能测试");
    
    let config = TestConfig::from_env();
    config.print_info();
    info!("🔄 性能测试模式：{} 次请求", config.test_count);

    // 创建HTTP客户端
    let proxy_url = format!("http://{}:{}", config.proxy_host, config.proxy_port);
    let proxy = Proxy::all(&proxy_url)?;
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(60))
        .build()?;

    // 执行测试
    let start_time = std::time::Instant::now();
    run_tests(&client, &config.target_url, config.test_count).await?;
    let total_time = start_time.elapsed();
    
    info!("⏱️  性能测试完成，总耗时: {:?}", total_time);
    info!("📊 平均每次请求耗时: {:?}", total_time / config.test_count as u32);

    Ok(())
}

#[tokio::test]
async fn test_proxy_python_like() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🐍 开始Python风格代理服务器测试");
    
    let config = TestConfig::from_env();
    config.print_info();

    // 创建HTTP客户端，模拟Python requests的行为
    let proxy_url = format!("http://{}:{}", config.proxy_host, config.proxy_port);
    let mut proxy = Proxy::all(&proxy_url)?;
    
    // 暂时注释掉认证，先测试基本代理功能
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        proxy = proxy.basic_auth(username, password);
    }
    
    let client = Client::builder()
        .proxy(proxy)
        .user_agent("python-requests/2.31.0")
        .timeout(Duration::from_secs(60))  // 更长的超时
        .danger_accept_invalid_certs(true)
        .connection_verbose(true)  // 启用连接详细日志
        .http2_prior_knowledge()              // 强制使用HTTP/1.1但禁用HTTP/2
        .build()?;

    info!("🧪 执行单次Python风格测试");
    
    let start_time = std::time::Instant::now();
    match timeout(Duration::from_secs(60), client.get(&config.target_url).send()).await {
        Ok(Ok(response)) => {
            let response_time = start_time.elapsed();
            let status = response.status();
            
            info!("📥 响应状态: {}", status);
            info!("⏱️  响应时间: {:?}", response_time);
            
            if status.is_success() {
                let content_length = response.content_length().unwrap_or(0);
                info!("📄 响应大小: {} bytes", content_length);
                
                let response_text = response.text().await?;
                info!("✅ Python风格测试成功！收到 {} 字符", response_text.len());
                Ok(())
            } else {
                Err(format!("HTTP请求失败，状态码: {}", status).into())
            }
        }
        Ok(Err(e)) => {
            error!("❌ Python风格测试请求失败: {}", e);
            Err(e.into())
        }
        Err(_) => {
            error!("❌ Python风格测试超时");
            Err("请求超时".into())
        }
    }
}

#[tokio::test]
async fn test_proxy_custom() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始自定义代理服务器测试");
    
    let config = TestConfig::from_env();
    config.print_info();

    // 创建HTTP客户端
    let proxy_url = format!("http://{}:{}", config.proxy_host, config.proxy_port);
    let mut proxy = Proxy::all(&proxy_url)?;
    
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        proxy = proxy.basic_auth(username, password);
    }
    
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(30))
        .user_agent("python-requests/2.31.0")  // 模拟Python requests的User-Agent
        .danger_accept_invalid_certs(true)     // 更宽松的证书验证
        .build()?;

    // 执行测试
    run_tests(&client, &config.target_url, config.test_count).await?;

    Ok(())
}

async fn run_tests(client: &Client, target_url: &str, test_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 开始执行 {} 次请求测试", test_count);
    
    let mut success_count = 0;
    let mut error_count = 0;
    let mut total_response_time = Duration::new(0, 0);

    for i in 1..=test_count {
        info!("📤 执行第 {}/{} 次请求", i, test_count);
        
        match test_single_request(client, target_url).await {
            Ok(response_time) => {
                success_count += 1;
                total_response_time += response_time;
                info!("✅ 第 {} 次请求成功，响应时间: {:?}", i, response_time);
            }
            Err(e) => {
                error_count += 1;
                error!("❌ 第 {} 次请求失败: {}", i, e);
            }
        }

        // 在请求之间添加小延迟
        if i < test_count {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // 输出测试结果统计
    info!("📊 测试结果统计:");
    info!("✅ 成功: {}/{}", success_count, test_count);
    info!("❌ 失败: {}/{}", error_count, test_count);
    
    let success_rate = (success_count as f64 / test_count as f64) * 100.0;
    info!("📈 成功率: {:.1}%", success_rate);
    
    if success_count > 0 {
        let avg_response_time = total_response_time / success_count as u32;
        info!("⏱️  平均响应时间: {:?}", avg_response_time);
    }

    if success_count == test_count {
        info!("🎉 所有测试通过！代理服务器工作正常");
        Ok(())
    } else if success_count > 0 {
        warn!("⚠️  部分测试通过，代理服务器可能存在问题");
        Err(format!("部分测试失败: {}/{} 成功", success_count, test_count).into())
    } else {
        error!("💥 所有测试失败，代理服务器无法正常工作");
        Err("所有测试失败".into())
    }
}

async fn test_single_request(client: &Client, target_url: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    
    // 发送HTTP请求，设置超时
    let response = timeout(Duration::from_secs(30), client.get(target_url).send()).await??;
    
    let status = response.status();
    let response_time = start_time.elapsed();
    
    info!("📥 响应状态: {}", status);
    info!("⏱️  响应时间: {:?}", response_time);
    
    // 检查响应状态码
    if status.is_success() {
        let content_length = response.content_length().unwrap_or(0);
        info!("📄 响应大小: {} bytes", content_length);
        
        // 读取部分响应内容以验证数据传输
        let response_text = response.text().await?;
        if !response_text.is_empty() {
            info!("📝 收到响应内容 (前100字符): {}", 
                &response_text[..response_text.len().min(100)]);
        }
        Ok(response_time)
    } else {
        Err(format!("HTTP请求失败，状态码: {}", status).into())
    }
}