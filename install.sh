#!/bin/bash

echo "========================================"
echo "API Key Vault - 开发环境安装脚本"
echo "========================================"
echo

# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "[错误] 未找到 Node.js"
    echo
    echo "请先安装 Node.js:"
    echo "  - macOS: brew install node"
    echo "  - Ubuntu/Debian: sudo apt install nodejs npm"
    echo "  - 或访问 https://nodejs.org/"
    echo
    exit 1
fi

echo "[√] Node.js 已安装: $(node --version)"

# 检查 pnpm
if ! command -v pnpm &> /dev/null; then
    echo
    echo "[!] pnpm 未安装，正在安装..."
    npm install -g pnpm
    if [ $? -ne 0 ]; then
        echo "[错误] pnpm 安装失败"
        exit 1
    fi
fi

echo "[√] pnpm 已安装: $(pnpm --version)"

echo
echo "[1/2] 安装前端依赖..."
pnpm install
if [ $? -ne 0 ]; then
    echo "[错误] 依赖安装失败"
    exit 1
fi

echo
echo "[2/2] 安装 Tauri CLI..."
pnpm add -D @tauri-apps/cli@latest
if [ $? -ne 0 ]; then
    echo "[错误] Tauri CLI 安装失败"
    exit 1
fi

echo
echo "========================================"
echo "安装完成！"
echo "========================================"
echo
echo "启动开发服务器:"
echo "  pnpm tauri dev"
echo
echo "构建生产版本:"
echo "  pnpm tauri build"
echo
