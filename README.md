# RustProxy

一个高性能的HTTP/HTTPS/WebSocket代理服务器，使用Rust编写。

## 功能特性

- ✅ 支持HTTP和HTTPS代理
- ✅ 支持HTTP/1.0、HTTP/1.1和HTTP/2协议
- ✅ 支持WebSocket代理
- ✅ 基于tokio的高性能异步I/O
- ✅ 支持HTTP基本认证
- ✅ 显式代理模式（目标服务器看到代理IP，保护客户端隐私）
- ✅ 可配置的并发连接数限制
- ✅ 详细的客户端连接日志追踪
- ✅ 模块化设计
- ✅ 命令行配置

## 安装

```bash
git clone <repository>
cd RustProxy
cargo build --release
```

## 使用方法

### 基本使用

启动默认配置的代理服务器（监听0.0.0.0:24975）：

```bash
cargo run --release
```

### 自定义配置

指定IP和端口：

```bash
cargo run --release -- --ip 127.0.0.1 --port 8080
```

### 启用认证

设置用户名和密码：

```bash
cargo run --release -- --username myuser --password mypass
```

### 设置并发连接数限制

限制最大并发连接数为500：

```bash
cargo run --release -- --max-connections 500
```

### 完整参数示例

```bash
cargo run --release -- --ip 192.168.1.100 --port 3128 --username admin --password secret --max-connections 1000
```

## 命令行参数

| 参数 | 短参数 | 描述 | 默认值 |
|------|--------|------|--------|
| `--ip` | `-i` | 监听IP地址 | `0.0.0.0` |
| `--port` | `-p` | 监听端口 | `24975` |
| `--username` | `-u` | 认证用户名 | 无 |
| `--password` | `-w` | 认证密码 | 无 |
| `--max-connections` | `-c` | 最大并发连接数 | `1000` |

## 客户端配置

### 无认证代理

将代理设置为：`<服务器IP>:24975`

### 认证代理

1. 代理地址：`<服务器IP>:24975`
2. 用户名：`-u` 参数指定的用户名
3. 密码：`-w` 参数指定的密码

### 浏览器示例

在浏览器中设置代理：
- HTTP代理：`22.33.44.55:24975`
- HTTPS代理：`22.33.44.55:24975`

如果启用了认证，还需要提供用户名和密码。

## 日志功能

代理服务器提供详细的日志记录，包含客户端连接信息：

```
INFO  🔓 代理服务器: 0.0.0.0:24975 (最大连接数: 1000)
INFO  接受新连接来自: 192.168.1.100:54321
INFO  [192.168.1.100:54321] 收到 HTTP 请求到 example.com:80
INFO  [192.168.1.100:54321] 成功连接到目标服务器 example.com:80
DEBUG [192.168.1.100:54321] 客户端到目标服务器流结束
```

所有日志都包含客户端地址 `[IP:端口]`，便于追踪和调试：

- 连接建立和断开
- 认证状态
- 请求类型和目标
- 错误信息
- 数据传输状态

## 安全注意事项

1. 在生产环境中请使用强密码
2. 建议在防火墙中限制对代理端口的访问
3. 根据服务器性能合理设置最大并发连接数
4. 定期检查访问日志，监控异常连接
5. 可通过日志追踪客户端行为进行安全审计

## 开发

项目结构：

```
src/
├── main.rs               # 主程序入口
├── lib.rs                # 库入口
├── config.rs             # 配置管理
├── auth.rs               # 认证模块
├── proxy.rs              # 代理核心逻辑
├── connection.rs         # 连接处理
├── parser/              # 协议解析
│   ├── mod.rs
│   └── detector.rs      # 协议检测
└── handlers/            # 协议处理器
    ├── mod.rs
    ├── backend.rs       # 后端连接器
    ├── http1.rs        # HTTP/1.x处理
    ├── http2.rs        # HTTP/2处理
    └── websocket.rs    # WebSocket处理
```

运行开发版本：

```bash
cargo run -- --ip 127.0.0.1 --port 8080 --username test --password test123 --max-connections 100
```

## 性能调优

### 并发连接数设置

- **低配置服务器**：100-500 连接
- **中等配置服务器**：500-2000 连接
- **高性能服务器**：2000+ 连接

建议根据以下因素调整：
- 服务器内存和CPU
- 网络带宽
- 预期客户端数量
- 平均连接持续时间

### 监控建议

1. 监控日志中的连接数量
2. 观察内存使用情况
3. 检查响应延迟
4. 跟踪拒绝连接的频率

## 协议支持详情

### HTTP/1.0 和 HTTP/1.1
- 完整支持GET、POST、PUT、DELETE等方法
- 支持Keep-Alive连接
- 自动解析Host头和路径
- 支持代理认证

### HTTP/2
- 支持HTTP/2 clear-text模式
- 自动检测HTTP/2 preface
- 流复用和帧转发
- 通过CONNECT隧道支持HTTP/2 over TLS

### WebSocket
- 自动检测WebSocket升级请求
- 支持ws://和wss://协议
- 透明转发WebSocket帧
- 处理Ping/Pong心跳

### 显式代理特性
- 所有协议使用显式代理模式
- 目标服务器只能看到代理服务器IP
- 不泄露客户端真实IP地址
- 支持标准Proxy-Authorization认证

## 性能特性

- 异步非阻塞I/O (tokio)
- 高效的内存使用
- 可配置的并发连接数限制
- 零拷贝数据转发
- 大缓冲区支持(8KB)

## 测试

运行多协议测试脚本：

```bash
./test_protocols.sh
```

或手动测试：

```bash
# 测试HTTP/1.1
curl -x http://username:password@localhost:8080 http://httpbin.org/ip

# 测试HTTPS (CONNECT隧道)
curl -x http://username:password@localhost:8080 https://httpbin.org/ip

# 测试WebSocket
wscat -c ws://localhost:8080 -H "Proxy-Authorization: Basic <base64>"
```

## 开发路线图

- [ ] HTTP/3 (QUIC) 支持
- [ ] 连接池优化
- [ ] 请求/响应日志记录
- [ ] 访问控制列表(ACL)
- [ ] 速率限制
- [ ] 监控指标导出

## 许可证

MIT License