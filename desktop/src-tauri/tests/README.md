# Picker Desktop Application (Tauri Backend) 测试文档

## 测试概述

本项目包含完整的测试套件，用于验证 Picker Desktop Application (Tauri Backend) 的功能正确性和稳定性。测试套件包括单元测试和集成测试两部分。

## 测试结构

测试文件组织如下：

```
src/tests/
├── api_test.rs        # API 模块测试
├── commands_test.rs   # 命令模块测试
├── config_test.rs     # 配置模块测试
└── utils_test.rs      # 工具模块测试
tests/
└── integration_test.rs  # 集成测试
```

## 测试内容

### 单元测试

1. **API 模块测试**
   - 测试 API 客户端的 GET、POST 请求功能
   - 测试错误处理和认证功能
   - 测试查询参数处理

2. **命令模块测试**
   - 测试用户相关命令：登录、注册、验证邮箱、获取用户资料
   - 测试错误处理和边界条件

3. **配置模块测试**
   - 测试从环境变量加载配置
   - 测试从配置文件加载配置
   - 测试默认配置和错误处理

4. **工具模块测试**
   - 测试认证管理器的基本功能
   - 测试 JWT token 解析和过期检查

### 集成测试

- **用户认证流程测试**：测试完整的用户注册、验证、登录、获取资料、登出流程
- **Picker 市场流程测试**：测试浏览市场、查看详情等功能
- **订单流程测试**：测试创建订单、查看订单列表和详情等功能

## 运行测试

### 运行所有测试

```bash
cd /opt/rust/project/picker/desktop/src-tauri
cargo test
```

### 运行特定测试

```bash
# 运行单元测试
cargo test --test api_test
cargo test --test commands_test
cargo test --test config_test
cargo test --test utils_test

# 运行集成测试
cargo test --test integration_test
```

### 运行单个测试函数

```bash
cargo test test_login_command -- --exact
```

### 查看测试覆盖率

安装 cargo-tarpaulin 工具：

```bash
cargo install cargo-tarpaulin
```

生成覆盖率报告：

```bash
cargo tarpaulin --out Html
```

## 测试注意事项

1. 测试使用 mockito 库模拟 API 响应，避免对真实服务器的依赖
2. 测试过程中可能会创建临时文件，测试完成后会自动清理
3. 集成测试会测试完整的功能流程，确保各模块协同工作正常
4. 建议在提交代码前运行所有测试，确保代码质量

## 测试环境要求

- Rust 1.77.2 或更高版本
- Cargo 包管理器
- 足够的磁盘空间用于测试临时文件

## 更新测试

当项目代码发生变更时，应相应更新测试代码以确保测试覆盖最新的功能和修复。特别是：

- 添加新功能时，应添加相应的单元测试和集成测试
- 修改现有功能时，应更新相关测试以反映变更
- 修复 bug 时，应添加测试以验证修复效果并防止回归