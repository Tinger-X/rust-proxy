# Rust 代理服务器黑盒测试指南

## 概述

本项目使用标准的Rust集成测试框架，支持通过 `.env` 文件配置测试参数。测试代码位于 `tests/black_test.rs` 文件中。

## 项目结构

```
rust-proxy/
├── src/                   # 代理服务器源代码
├── tests/                 # 集成测试目录
│   ├── black_test.rs      # 黑盒测试代码
│   ├── .env.example       # 配置文件模板
│   ├── .env               # 实际配置文件（已git忽略）
│   └── guide.md           # 本测试指南
├── .gitignore             # Git忽略文件
├── Cargo.toml             # 项目依赖（包含测试依赖）
└── switch_config.ps1      # 配置切换脚本
```

## 🚀 快速开始

### 第一步：配置测试参数

1. **复制配置模板**：
   ```powershell
   cp tests/.env.example tests/.env
   ```

2. **编辑配置文件**：
   ```powershell
   notepad tests/.env
   ```

   根据你的需求修改配置：
   ```env
   # 代理服务器配置
   PROXY_HOST=127.0.0.1
   PROXY_PORT=24975
   
   # 测试目标配置
   TARGET_URL=https://www.baidu.com
   TEST_COUNT=10
   
   # 认证配置（可选）
   PROXY_USERNAME=test
   PROXY_PASSWORD=test123
   ```

### 第二步：启动代理服务器

打开**第一个终端**，手动启动代理服务器：

#### 无认证模式
```powershell
cd "d:/Project/Rust/rust-proxy"
cargo run --release -- --ip 127.0.0.1 --port 24975 --max-connections 100
```

#### 认证模式
```powershell
cd "d:/Project/Rust/rust-proxy"
cargo run --release -- --ip 127.0.0.1 --port 24975 --username test --password test123 --max-connections 100
```

### 第三步：运行测试

打开**第二个终端**，运行测试：

```powershell
cd "d:/Project/Rust/rust-proxy"

# 运行所有测试
cargo test --test black_test -- --nocapture

# 运行特定测试
cargo test --test black_test test_proxy_custom -- --nocapture
```

## 📝 配置文件

### .env 文件格式

```env
# 代理服务器配置
PROXY_HOST=127.0.0.1
PROXY_PORT=24975

# 测试目标配置
TARGET_URL=https://www.baidu.com
TEST_COUNT=10

# 认证配置（可选）
PROXY_USERNAME=test
PROXY_PASSWORD=test123
```

### 配置参数说明

| 参数 | 描述 | 默认值 | 示例 |
|------|------|--------|------|
| `PROXY_HOST` | 代理服务器地址 | `127.0.0.1` | `192.168.1.100` |
| `PROXY_PORT` | 代理服务器端口 | `24975` | `8080` |
| `TARGET_URL` | 目标测试URL | `https://www.baidu.com` | `https://www.google.com` |
| `TEST_COUNT` | 测试请求次数 | `10` | `50` |
| `PROXY_USERNAME` | 认证用户名 | 无 | `myuser` |
| `PROXY_PASSWORD` | 认证密码 | 无 | `mypass` |

## 🧪 测试函数

### 可用测试

1. **`test_proxy_without_auth`** - 无认证模式测试
2. **`test_proxy_with_auth`** - 认证模式测试
3. **`test_proxy_http_target`** - HTTP目标测试
4. **`test_proxy_performance`** - 性能测试
5. **`test_proxy_custom`** - 自定义配置测试（推荐）

## 📋 配置示例

### 示例1：无认证测试

编辑 `.env` 文件：
```env
PROXY_HOST=127.0.0.1
PROXY_PORT=24975
TARGET_URL=https://www.baidu.com
TEST_COUNT=10
# PROXY_USERNAME=
# PROXY_PASSWORD=
```

运行测试：
```powershell
cargo test --test black_test test_proxy_custom -- --nocapture
```

### 示例2：认证测试

编辑 `.env` 文件：
```env
PROXY_HOST=127.0.0.1
PROXY_PORT=24975
TARGET_URL=https://www.baidu.com
TEST_COUNT=10
PROXY_USERNAME=test
PROXY_PASSWORD=test123
```

运行测试：
```powershell
cargo test --test black_test test_proxy_custom -- --nocapture
```

### 示例3：HTTP目标测试

编辑 `.env` 文件：
```env
PROXY_HOST=127.0.0.1
PROXY_PORT=24975
TARGET_URL=http://httpbin.org/get
TEST_COUNT=5
```

运行测试：
```powershell
cargo test --test black_test test_proxy_custom -- --nocapture
```

### 示例4：性能测试

编辑 `.env` 文件：
```env
PROXY_HOST=127.0.0.1
PROXY_PORT=24975
TARGET_URL=https://www.baidu.com
TEST_COUNT=50
```

运行测试：
```powershell
cargo test --test black_test test_proxy_performance -- --nocapture
```

### 示例5：多目标测试

#### 测试百度
编辑 `.env` 文件：
```env
PROXY_HOST=127.0.0.1
PROXY_PORT=24975
TARGET_URL=https://www.baidu.com
TEST_COUNT=10
PROXY_USERNAME=test
PROXY_PASSWORD=test123
```

#### 测试Google
修改 `.env` 文件：
```env
TARGET_URL=https://www.google.com
```

#### 测试API接口
修改 `.env` 文件：
```env
TARGET_URL=https://api.github.com/users
TEST_COUNT=5
```

## 🎯 使用技巧

### 1. 快速切换配置

创建多个配置文件：
```powershell
# 基础测试
copy tests/.env tests/.env.basic

# 性能测试
copy tests/.env tests/.env.performance

# API测试
copy tests/.env tests/.env.api
```

然后快速切换：
```powershell
# 使用基础配置
copy tests/.env.basic tests/.env
cargo test --test black_test test_proxy_custom

# 使用性能测试配置
copy tests/.env.performance tests/.env
cargo test --test black_test test_proxy_performance
```

### 2. 临时覆盖配置

如果你想要临时使用不同的配置而不修改 `.env` 文件，仍然可以使用环境变量：

```powershell
# 临时使用不同的测试次数
$env:TEST_COUNT="20"
cargo test --test black_test test_proxy_custom

# 临时测试不同的目标
$env:TARGET_URL="https://www.google.com"
cargo test --test black_test test_proxy_custom
```

### 3. 配置验证

运行测试前可以查看当前配置：
```powershell
Get-Content tests/.env
```

## 📊 预期输出

### 成功的测试输出

```
INFO  ✅ 已加载.env文件配置
INFO  🚀 开始自定义代理服务器测试
INFO  📡 代理服务器: 127.0.0.1:24975
INFO  🎯 目标URL: https://www.baidu.com
INFO  🔄 测试次数: 10
INFO  🔐 认证用户: test
INFO  📄 配置来源: .env文件或环境变量
INFO  🧪 开始执行 10 次请求测试
INFO  📤 执行第 1/10 次请求
INFO  📥 响应状态: 200 OK
INFO  ⏱️  响应时间: 245ms
INFO  ✅ 第 1 次请求成功，响应时间: 245ms
...
INFO  📊 测试结果统计:
INFO  ✅ 成功: 10/10
INFO  ❌ 失败: 0/10
INFO  📈 成功率: 100.0%
INFO  🎉 所有测试通过！代理服务器工作正常
```

### 代理服务器端日志

```
INFO  🔒 代理服务器: 127.0.0.1:24975 (最大连接数: 100)
INFO  接受新连接来自: 127.0.0.1:54321
INFO  [127.0.0.1:54321] 收到 CONNECT 请求到 www.baidu.com:443
INFO  [127.0.0.1:54321] 成功连接到目标服务器 www.baidu.com:443
```

## 🔧 故障排除

### 常见问题

1. **.env文件未加载**
   - 确认 `tests/.env` 文件存在
   - 检查文件格式是否正确（没有BOM头）
   - 查看测试输出中的加载信息

2. **配置参数未生效**
   - 检查参数名称是否正确
   - 确认没有多余的空格或特殊字符
   - 验证参数值格式（如端口号必须是数字）

3. **认证失败**
   - 确认 `.env` 中的用户名和密码正确
   - 确保代理服务器以认证模式启动
   - 检查用户名和密码前后没有空格

4. **连接被拒绝**
   - 检查代理服务器是否运行
   - 验证 `PROXY_HOST` 和 `PROXY_PORT` 设置
   - 确认防火墙设置

### 调试技巧

1. **查看当前配置加载情况**：
   ```powershell
   cargo test --test black_test test_proxy_custom -- --nocapture | grep "配置来源"
   ```

2. **测试单一请求**：
   修改 `.env` 文件：
   ```env
   TEST_COUNT=1
   ```

3. **检查.env文件格式**：
   ```powershell
   Get-Content tests/.env | Where-Object { $_ -match "=" }
   ```

## 🛡️ 安全注意事项

1. **不要提交tests/.env文件到Git**
   - `tests/.env` 文件已添加到 `.gitignore`
   - 包含敏感信息如密码

2. **使用.env.example作为模板**
   - 提供配置示例
   - 不包含真实敏感信息

3. **生产环境注意事项**
   - 使用强密码
   - 定期更换认证信息
   - 限制代理服务器访问

## 📈 性能优化建议

### 调整测试参数

```env
# 快速测试
TEST_COUNT=5

# 标准测试
TEST_COUNT=10

# 性能测试
TEST_COUNT=50

# 压力测试
TEST_COUNT=100
```

### 网络优化

- 使用本地代理服务器进行测试
- 选择响应较快的目标URL
- 合理设置测试间隔

## 🎉 推荐工作流程

1. **初始设置**：
   ```powershell
   cp tests/.env.example tests/.env
   # 编辑 tests/.env 文件
   ```

2. **日常测试**：
   ```powershell
   # 根据需要编辑 tests/.env
   cargo test --test black_test test_proxy_custom -- --nocapture
   ```

3. **特定测试**：
   ```powershell
   # 性能测试
   cargo test --test black_test test_proxy_performance

   # 无认证测试
   cargo test --test black_test test_proxy_without_auth
   ```

4. **批量测试**：
   创建脚本自动切换不同配置进行测试

现在你可以通过简单的 `.env` 文件配置来管理所有测试参数了！