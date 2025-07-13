#!/bin/bash

# DEX Events Parser Runner Script

echo "🚀 Starting DEX Events Parser..."

# Kiểm tra file .env
if [ ! -f .env ]; then
    echo "⚠️  File .env không tồn tại. Tạo file .env với nội dung sau:"
    echo ""
    echo "# RPC WebSocket URL cho Solana Mainnet"
    echo "RPC_WS_URL=wss://api.mainnet-beta.solana.com"
    echo ""
    echo "# Log level (trace, debug, info, warn, error)"
    echo "RUST_LOG=info"
    echo ""
    echo "Sau khi tạo file .env, chạy lại script này."
    exit 1
fi

# Set environment variables
export RUST_LOG=info
export RUST_BACKTRACE=1

# Build và chạy service
echo "🔨 Building service..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build thành công!"
    echo "🎯 Đang kết nối và parse DEX events..."
    echo ""
    echo "Các event sẽ được hiển thị theo format:"
    echo "[EVENT_TYPE] [PLATFORM] [SIGNATURE] [DETAILS]"
    echo ""
    echo "Nhấn Ctrl+C để dừng service."
    echo ""
    
    # Chạy service
    cargo run --release
else
    echo "❌ Build thất bại!"
    exit 1
fi 