# Cmirror (China Mirror Manager)

🇨🇳 **专为中国大陆开发者打造的一键换源工具**

Cmirror 是一个基于 Rust 编写的跨平台命令行工具，旨在解决国内开发环境依赖下载速度慢、配置繁琐的问题。它提供“并发测速-对比-自动配置”的一站式解决方案，支持 pip, npm, docker 等多种常见开发工具。

## ✨ 核心功能

* **⚡️ 极速体验**: 使用 HTTP/HTTPS `HEAD` 请求并发测试所有镜像源延迟，精准计算 TTFB (Time To First Byte)。
* **🛡️ 安全无忧**: 修改任何配置前强制自动备份，支持一键恢复 (`restore`)。
* **🧠 智能推荐**: 支持 `--fastest` 参数，自动选择并应用当前网络环境下最快的源。
* **📊 状态透视**: 一目了然地查看当前所有工具正在使用的源地址及状态。

## 📦 支持列表

| 工具 | 状态 | 配置文件路径 | 备注 |
| :--- | :--- | :--- | :--- |
| **pip** (Python) | ✅ 支持 | `~/.pip/pip.conf` (Linux/Mac) | 支持 venv 及全局配置 |
| **uv** (Python) | ✅ 支持 | `uv.toml` | 优先项目级配置，其次全局 |
| **conda** (Python) | ✅ 支持 | `~/.condarc` | 自动配置 channels |
| **npm** (Node.js) | ✅ 支持 | `~/.npmrc` | |
| **docker** | ✅ 支持 | `/etc/docker/daemon.json` | 需 sudo 权限 |
| **apt** (Ubuntu/Debian) | ✅ 支持 | `/etc/apt/sources.list` | 智能替换域名，需 sudo |
| **cargo** (Rust) | ✅ 支持 | `~/.cargo/config.toml` | 替换 crates.io 索引 |
| **go** (Golang) | ✅ 支持 | 环境变量 (`GOPROXY`) | 使用 `go env` 管理 |
| **brew** (Homebrew) | ✅ 支持 | 环境变量 | 提供 export 命令提示 |

## 🚀 安装指南

### 方式一：下载预编译二进制文件（推荐）

从 [Releases](https://github.com/ox01024/cmirror/releases) 页面下载适合你系统的压缩包：

**Linux (x86_64):**
```bash
# 下载并解压
wget https://github.com/ox01024/cmirror/releases/latest/download/cmirror-linux-x64.tar.gz
tar -xzf cmirror-linux-x64.tar.gz

# 添加执行权限
chmod +x cmirror

# 移动到系统路径（可选）
sudo mv cmirror /usr/local/bin/

# 验证安装
cmirror --help
```

**Linux (ARM64):**
```bash
# 下载并解压
wget https://github.com/ox01024/cmirror/releases/latest/download/cmirror-linux-arm64.tar.gz
tar -xzf cmirror-linux-arm64.tar.gz

# 添加执行权限
chmod +x cmirror

# 移动到系统路径（可选）
sudo mv cmirror /usr/local/bin/

# 验证安装
cmirror --help
```

**macOS (Apple Silicon):**
```bash
# 下载并解压
curl -L -o cmirror-macos-arm64.tar.gz https://github.com/ox01024/cmirror/releases/latest/download/cmirror-macos-arm64.tar.gz
tar -xzf cmirror-macos-arm64.tar.gz

# 添加执行权限
chmod +x cmirror

# 移动到系统路径（可选）
sudo mv cmirror /usr/local/bin/

# 验证安装
cmirror --help
```

**macOS (Intel):**
```bash
# 下载并解压
curl -L -o cmirror-macos-x64.tar.gz https://github.com/ox01024/cmirror/releases/latest/download/cmirror-macos-x64.tar.gz
tar -xzf cmirror-macos-x64.tar.gz

# 添加执行权限
chmod +x cmirror

# 移动到系统路径（可选）
sudo mv cmirror /usr/local/bin/

# 验证安装
cmirror --help
```

**Windows:**
```powershell
# 下载压缩包
Invoke-WebRequest -Uri "https://github.com/ox01024/cmirror/releases/latest/download/cmirror-windows-x64.zip" -OutFile "cmirror.zip"

# 解压
Expand-Archive -Path cmirror.zip -DestinationPath .

# 将 cmirror.exe 添加到 PATH 环境变量，或移动到已在 PATH 中的目录

# 验证安装
.\cmirror.exe --help
```

### 方式二：源码编译安装

**前置要求：** 确保你已经安装了 Rust 工具链 (Cargo)。

```bash
# 1. 克隆仓库
git clone https://github.com/ox01024/cmirror.git
cd cmirror

# 2. 编译并安装
cargo install --path .

# 3. 验证安装
cmirror --help
```

## 📖 使用文档

### 1. 查看当前状态

查看所有支持工具的当前配置源：

```bash
$ cmirror status

Tool       Current Source URL                       Status
----------------------------------------------------------------------
pip        Default                                  [Official/Default]
npm        https://registry.npmmirror.com           [Taobao]
docker     Default                                  [Official/Default]
go         https://proxy.golang.org                 [Official]
cargo      Default                                  [Official/Default]
brew       Default                                  [Official/Default]
----------------------------------------------------------------------
```

也可以只查看特定工具：`cmirror status pip`

### 2. 测速对比

测试并列出可用镜像源的延迟排名：

```bash
$ cmirror test pip

Benchmarking 6 mirrors for pip...
[||||||||||||||||||||||||||||||||||||||||] 100% Testing completed.

RANK  LATENCY    NAME         URL
------------------------------------------------------------
1     25ms       Aliyun       https://mirrors.aliyun.com/pypi/simple/
2     38ms       Tuna         https://pypi.tuna.tsinghua.edu.cn/simple
3     900ms      Official     https://pypi.org/simple
------------------------------------------------------------
Recommendation: 'Aliyun' is 36x faster than your current source.
Run 'cmirror use pip Aliyun' to apply.
```

### 3. 切换镜像源

**自动选择最快源 (推荐):**

```bash
cmirror use pip --fastest
```

**指定特定源:**

```bash
cmirror use pip aliyun
```

*注意：修改 Docker, Apt 等系统级配置时可能需要 root 权限，请使用 `sudo cmirror use docker ...`*

### 4. 恢复配置 (Restore)

如果需要回滚到上一次的配置（或重置为默认）：

```bash
cmirror restore pip
```

*(注：对于 pip, npm, docker, cargo, apt，这将恢复最近的 `.bak` 备份文件；对于 go, brew，将重置或提示取消环境变量)*

## 🛠️ 开发计划 (Roadmap)

* [x] 基础 CLI 框架 (Status, Test, Use)
* [x] 支持 pip, npm
* [x] 支持 Docker (Linux/macOS)
* [x] 支持 apt (Ubuntu/Debian)
* [x] 支持 Rust Cargo, Go Modules
* [x] 支持 Homebrew (Env hint)
* [x] `restore` 灾难恢复命令
* [ ] 支持 yum/dnf (CentOS/Fedora)
* [ ] TUI 交互式界面 (Dialoguer)

## 🤝 反馈与共建

> 🚧 **温馨提示**：受限于开发环境与测试设备，本项目在部分系统或特定配置下的表现可能尚未经过完全覆盖的严格测试。

如果您在使用过程中遇到任何“水土不服”或意外 Bug，请不吝前往 [Issues](https://github.com/ox01024/cmirror/issues) 提交反馈。您的每一次报错、建议或 PR，都是让 `cmirror` 变得更稳健的重要动力！

开源不易，感谢每一位开发者的包容与支持。❤️
