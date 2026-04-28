FROM rust:latest AS rust-base
ENV DEBIAN_FRONTEND=noninteractive

RUN chmod 777 /tmp
RUN apt update && apt -y upgrade

# Avoid select timezone
ENV DEBIAN_FRONTEND=noninteractive
ENV LANG="en_US.UTF-8"
ENV LANGUAGE="en_US:en"
ARG TZ=Asia/Shanghai
RUN apt -y install tzdata && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && apt -y install locales && locale-gen en_US.UTF-8 \
    && apt -y install lsb-release wget

RUN apt -y install wget vim git curl zip unzip trash-cli parallel libssl-dev iputils-ping \
    && apt -y install build-essential ninja-build gdb systemd-coredump nfs-common cmake \
    && apt -y install libgflags-dev libgoogle-glog-dev libgtest-dev libgmock-dev rsync git-lfs


RUN apt-get update && apt-get install -y \
    curl lld llvm clang gcc-mingw-w64 g++-mingw-w64 \
    pkg-config libssl-dev nsis libappindicator3-dev \
    && rm -rf /var/lib/apt/lists/*

# 安装 Node.js (例如 18)
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g pnpm

# 安装 Tauri CLI
RUN cargo install tauri-cli 

RUN rustup target add x86_64-pc-windows-gnu

RUN cargo install cargo-xwin

WORKDIR /workspace