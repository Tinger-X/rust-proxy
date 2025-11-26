# RustProxy

一个高性能的HTTP/HTTPS代理服务器，使用Rust编写。

## 功能特性

- ✅ 支持HTTP和HTTPS代理
- ✅ 基于tokio的高性能异步I/O
- ✅ 支持HTTP基本认证
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

### 完整参数示例

```bash
cargo run --release -- --ip 192.168.1.100 --port 3128 --username admin --password secret
```

## 命令行参数

| 参数 | 短参数 | 描述 | 默认值 |
|------|--------|------|--------|
| `--ip` | `-i` | 监听IP地址 | `0.0.0.0` |
| `--port` | `-p` | 监听端口 | `24975` |
| `--username` | `-u` | 认证用户名 | 无 |
| `--password` | `-w` | 认证密码 | 无 |

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

## 安全注意事项

1. 在生产环境中请使用强密码
2. 建议在防火墙中限制对代理端口的访问
3. 定期检查访问日志

## 开发

项目结构：

```
src/
├── main.rs          # 主程序入口
├── lib.rs           # 库入口
├── config.rs        # 配置管理
├── auth.rs          # 认证模块
├── proxy.rs         # 代理核心逻辑
└── connection.rs    # 连接处理
```

运行开发版本：

```bash
cargo run -- --ip 127.0.0.1 --port 8080 --username test --password test123
```

## 许可证

MIT License