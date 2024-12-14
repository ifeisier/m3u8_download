#!/bin/bash

# 通过 Docker 镜像 azureiotedge/armv7-unknown-linux-musleabihf:0.1 交叉编译


script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$script_dir/../" && pwd)"

echo "==============================="
echo "$script_dir"
echo "$root"
echo "==============================="

docker run --rm -it \
--cpus="2" \
-v "$root:/workspace/embedded-linux" \
azureiotedge/armv7-unknown-linux-musleabihf:0.1 \
/bin/bash -c "apt-get update && apt-get install -y curl git && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . '$HOME/.cargo/env' && source ~/.bashrc && cargo --version && \
    rustup target add armv7-unknown-linux-musleabihf && cd /workspace/embedded-linux && \
    rm Cargo.lock && \
    cargo clean && cargo build --target=armv7-unknown-linux-musleabihf --release"