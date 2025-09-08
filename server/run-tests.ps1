# Pickers Server 测试运行脚本
# 运行所有单元测试和集成测试

param(
    [switch]$Coverage,      # 是否生成代码覆盖率报告
    [switch]$Verbose,       # 详细输出
    [switch]$UnitOnly,      # 仅运行单元测试
    [switch]$IntegrationOnly, # 仅运行集成测试
    [string]$Filter = ""    # 测试过滤器
)

Write-Host "🧪 Pickers Server 测试套件" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Gray

# 检查 Rust 环境
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ 错误: 未找到 Cargo。请确保已安装 Rust。" -ForegroundColor Red
    exit 1
}

# 设置测试环境变量
$env:RUST_TEST_THREADS = "1"  # 避免数据库并发问题
$env:RUST_BACKTRACE = "1"     # 显示详细错误信息

# 构建测试参数
$testArgs = @()

if ($Verbose) {
    $testArgs += "--verbose"
}

if ($Filter) {
    $testArgs += $Filter
}

# 运行测试
try {
    if ($UnitOnly) {
        Write-Host "🔬 运行单元测试..." -ForegroundColor Yellow
        $testArgs += "--lib"
        & cargo test @testArgs
    }
    elseif ($IntegrationOnly) {
        Write-Host "🔗 运行集成测试..." -ForegroundColor Yellow
        $testArgs += "--test"
        $testArgs += "integration_tests"
        & cargo test @testArgs
    }
    else {
        Write-Host "🧪 运行所有测试..." -ForegroundColor Yellow
        & cargo test @testArgs
    }

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ 所有测试通过!" -ForegroundColor Green
        
        if ($Coverage) {
            Write-Host "📊 生成代码覆盖率报告..." -ForegroundColor Yellow
            
            # 检查是否安装了 tarpaulin
            if (-not (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue)) {
                Write-Host "⚠️  未找到 cargo-tarpaulin，正在安装..." -ForegroundColor Yellow
                & cargo install cargo-tarpaulin
            }
            
            # 生成覆盖率报告
            & cargo tarpaulin --out Html --output-dir coverage
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "📊 覆盖率报告已生成: coverage/tarpaulin-report.html" -ForegroundColor Green
                
                # 尝试打开报告
                if (Test-Path "coverage/tarpaulin-report.html") {
                    Start-Process "coverage/tarpaulin-report.html"
                }
            }
            else {
                Write-Host "❌ 覆盖率报告生成失败" -ForegroundColor Red
            }
        }
    }
    else {
        Write-Host "❌ 测试失败!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}
catch {
    Write-Host "❌ 测试运行出错: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎯 测试完成!" -ForegroundColor Cyan
Write-Host ""
Write-Host "💡 使用提示:" -ForegroundColor Gray
Write-Host "  .\run-tests.ps1                    # 运行所有测试" -ForegroundColor Gray
Write-Host "  .\run-tests.ps1 -UnitOnly          # 仅运行单元测试" -ForegroundColor Gray
Write-Host "  .\run-tests.ps1 -IntegrationOnly   # 仅运行集成测试" -ForegroundColor Gray
Write-Host "  .\run-tests.ps1 -Coverage          # 生成覆盖率报告" -ForegroundColor Gray
Write-Host "  .\run-tests.ps1 -Verbose           # 详细输出" -ForegroundColor Gray
Write-Host "  .\run-tests.ps1 -Filter 'test_name' # 运行特定测试" -ForegroundColor Gray