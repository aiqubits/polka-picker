## 自动化部署脚本






### 配置sqlite环境变量

```powershell
$envPath = [Environment]::GetEnvironmentVariable("Path", "User")
$newPath = "C:\Users\aiqubit\.pickers\sqlite"
if ($envPath -notlike "*$newPath*") {
    $envPath += ";$newPath"
    [Environment]::SetEnvironmentVariable("Path", $envPath, "User")
    Write-Host "路径已成功添加到用户PATH。"
} else {
    Write-Host "路径已存在于用户PATH中。"
}
```


