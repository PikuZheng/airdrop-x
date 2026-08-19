# AirDrop-X

AirDrop-X 是一个面向 Windows 与 Android 的开源、无服务器文件传输工具。设备通过 BLE 发现彼此，使用 Wi‑Fi Direct 建立无线直连网络，再通过 TCP 流式传输文件。

## 功能

- Windows 与 Android 双向 BLE 广播、扫描
- Android 创建 Wi‑Fi Direct Group，Windows 自动连接
- Windows ↔ Android 双向文件发送与接收
- Android 自定义接收目录并持久保存授权
- TCP 流式传输，不把整个文件读入内存
- SHA-256 完整性校验
- 临时文件校验成功后再改名，避免保留损坏文件
- Windows 单文件 GUI 程序，无控制台黑框
- 无账号、无云服务、无需互联网

## 使用流程

1. 在 Android 端允许“附近设备”和蓝牙权限。
2. Android 点击“选择接收目录”，然后点击“开启 Wi‑Fi Direct 并接收”。
3. Windows 点击“扫描附近设备”。
4. 发现手机后点击“通过 Wi‑Fi Direct 连接”。
5. 任意一端选择文件并发送。

两台设备必须开启蓝牙并支持 Wi‑Fi Direct。首次连接时 Windows 可能显示系统配对确认。

## 项目结构

```text
airdrop-x/
├── android/              Android 原生 Kotlin 客户端
├── protocol/             Rust ADX1 文件协议与测试
├── windows/              Tauri Windows 客户端与原生 Web UI
├── prd.md                产品需求文档
├── Cargo.toml            Rust 工作区
└── README.md
```

## 开发环境

- Windows 10/11
- Rust 1.85+
- Node.js 20+
- Android Studio、Android SDK 35、JDK 17+

项目已配置国内依赖源：

- npm：npmmirror
- Cargo：rsproxy
- Gradle：阿里云 Maven 镜像

## Windows 开发与构建

安装依赖并启动开发模式：

```powershell
cd windows
npm install
npm run tauri dev
```

构建内嵌前端资源、无黑框的单文件 EXE：

```powershell
cd windows
npm run tauri -- build --no-bundle
```

产物位于 `target/release/airdrop-x-windows.exe`。不要用普通的 `cargo build --release` 代替 Tauri 构建命令，否则前端可能仍指向开发服务器。

## Android 开发与构建

用 Android Studio 打开 `android` 目录，或使用 Gradle 构建：

```powershell
cd android
gradlew assembleDebug
```

APK 位于 `android/app/build/outputs/apk/debug/app-debug.apk`。

## ADX1 文件协议

每次 TCP 文件传输按顺序发送：

1. `ADX1`：4 字节魔数
2. 文件名长度：u16，大端序
3. UTF-8 文件名
4. 文件大小：u64，大端序
5. SHA-256：32 字节
6. 原始文件流

`ADXP` 用作 Wi‑Fi Direct 建链后的轻量握手，帮助 Android 自动获取 Windows 的直连地址。

## 测试

```powershell
cargo test --workspace
npm --prefix windows run build
```

Android 可通过 Android Studio 或 Gradle 的 `assembleDebug` 任务完成编译验证。

## V0.1 范围

当前版本实现 BLE 发现、Wi‑Fi Direct 建链、TCP 文件传输与完整性校验。QUIC、端到端加密和断点续传属于后续版本。

## 许可证

MIT
