@echo off
echo ========================================
echo API Key Vault - 开发环境安装脚本
echo ========================================
echo.

REM 检查 Node.js
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未找到 Node.js
    echo.
    echo 请先安装 Node.js:
    echo   1. 访问 https://nodejs.org/
    echo   2. 下载 LTS 版本
    echo   3. 运行安装程序
    echo.
    echo 安装完成后重新运行此脚本
    pause
    exit /b 1
)

echo [√] Node.js 已安装
node --version

REM 检查 pnpm
where pnpm >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [!] pnpm 未安装，正在安装...
    call npm install -g pnpm
    if %errorlevel% neq 0 (
        echo [错误] pnpm 安装失败
        pause
        exit /b 1
    )
)

echo [√] pnpm 已安装
pnpm --version

echo.
echo [1/2] 安装前端依赖...
call pnpm install
if %errorlevel% neq 0 (
    echo [错误] 依赖安装失败
    pause
    exit /b 1
)

echo.
echo [2/2] 安装 Tauri CLI...
call pnpm add -D @tauri-apps/cli@latest
if %errorlevel% neq 0 (
    echo [错误] Tauri CLI 安装失败
    pause
    exit /b 1
)

echo.
echo ========================================
echo 安装完成！
echo ========================================
echo.
echo 启动开发服务器:
echo   pnpm tauri dev
echo.
echo 构建生产版本:
echo   pnpm tauri build
echo.
pause
