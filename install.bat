@echo off
echo ========================================
echo API Key Vault - Windows 安装脚本
echo ========================================
echo.

REM 尝试查找 Node.js
where node >nul 2>&1
if %errorlevel% neq 0 (
    REM 检查便携版 Node.js
    if exist "C:\Users\%USERNAME%\nodejs\node-v22.16.0-win-x64\node.exe" (
        set "PATH=C:\Users\%USERNAME%\nodejs\node-v22.16.0-win-x64;%PATH%"
        echo [√] 找到便携版 Node.js
    ) else (
        echo [错误] 未找到 Node.js
        echo.
        echo 请安装 Node.js:
        echo   1. 访问 https://nodejs.org/
        echo   2. 下载 LTS 版本
        echo   3. 运行安装程序
        echo.
        pause
        exit /b 1
    )
)

echo [√] Node.js:
node --version

REM 检查 pnpm
where pnpm >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [!] pnpm 未安装，正在安装...
    call npm install -g pnpm
)

echo [√] pnpm:
pnpm --version

echo.
echo [1/2] 安装前端依赖...
call pnpm install --ignore-scripts
if %errorlevel% neq 0 (
    echo [错误] 依赖安装失败
    pause
    exit /b 1
)

echo.
echo ========================================
echo 安装完成！
echo ========================================
echo.
echo 启动应用:
echo   pnpm tauri dev
echo.
echo 仅运行 CLI:
echo   cargo run -p apikey-vault-cli -- --help
echo.
pause
