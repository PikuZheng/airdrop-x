```markdown
# AirDrop-X PRD 文档

## BLE + Wi-Fi Direct + QUIC 跨平台文件传输系统

版本：v1.0  
状态：Draft  
目标平台：

- Windows Desktop
- Android Mobile

---

# 1. 产品概述

## 1.1 产品名称

AirDrop-X

---

## 1.2 产品定位

打造一个开源、无服务器、无需互联网的 Windows ↔ Android 高速文件传输工具。

核心体验：

> 像 Apple AirDrop 一样，发现附近设备，确认后极速传输文件。

---

## 1.3 核心设计理念

采用三层无线架构：

```

BLE
↓
Wi-Fi Direct
↓
QUIC/TCP

```

职责：

|技术|作用|
|-|-|
|BLE|附近设备发现、身份交换、授权|
|Wi-Fi Direct|建立高速无线连接|
|QUIC/TCP|文件高速传输|

---

# 2. 产品目标

## 2.1 用户目标

用户可以：

- 手机发现附近 Windows 电脑
- Windows 发现附近 Android 手机
- 不需要互联网
- 不需要服务器
- 不需要登录账号
- 快速发送文件

---

## 2.2 性能目标

|指标|目标|
|-|-|
|设备发现时间|<3秒|
|连接建立|<5秒|
|传输速度|100MB/s+（取决于硬件）|
|CPU占用|<10%|
|内存占用|<200MB|
|支持文件大小|无限制|

---

# 3. 使用场景

## 场景1：手机发送文件到电脑

例如：

用户拍摄视频：

```

Android
|
|
AirDrop-X
|
|
Windows

```

发送：

- 视频
- 图片
- 文档


---

## 场景2：电脑发送文件到手机

例如：

电脑下载电影：

```

Windows

↓

Android

```

---

## 场景3：无网络环境

例如：

- 飞机
- 户外
- 无WiFi环境

无需：

- 路由器
- 云服务器
- 数据线

---

# 4. 系统架构

## 4.1 总体架构


```

```
             用户界面

    Windows UI       Android UI


         ↓


      Session Manager


         ↓
```

┌───────────┬────────────┐
│           │            │
BLE Layer  WiFi Layer  Transfer Layer

│           │            │

BLE       WiFi Direct    QUIC

发现       建链          文件传输

````


---

# 5. 核心流程设计


# 5.1 设备发现流程


## BLE广播

Android:

广播：

```json
{
 device:"Pixel",
 service:"AirDrop-X",
 version:"1.0",
 features:[
   "wifi-direct",
   "quic"
 ]
}
````

Windows:

扫描：

```
发现:

Pixel

距离:
3m

支持:
WiFi Direct
QUIC
```

---

# 5.2 身份认证流程

发现设备后：

交换：

* Device ID
* Public Key
* Capability

流程：

```
Android

生成密钥

      ↓

BLE交换公钥

      ↓

ECDH

      ↓

生成Session Key

```

加密：

```
AES-256-GCM
```

---

# 5.3 Wi-Fi Direct连接流程

## 角色选择

双方协商：

```
Android
  |
  | Group Owner
  |
Windows Client
```

或者：

```
Windows
  |
  | Group Owner
  |
Android Client
```

---

连接成功：

生成局域网：

```
Android

192.168.49.1


Windows

192.168.49.20

```

---

# 5.4 文件传输流程

## 建立QUIC连接

```
Android

QUIC Client


       ↓


Windows

QUIC Server

```

---

发送文件信息：

```json
{
 filename:"video.mp4",
 size:2048000000,
 hash:"sha256xxx"
}
```

---

文件切片：

默认：

```
4MB/chunk
```

例如：

```
file

chunk0
chunk1
chunk2
chunk3

```

---

传输：

```
Chunk

 ↓

AES Encrypt

 ↓

QUIC Stream

 ↓

Receiver

 ↓

Verify Hash

```

---

# 6. 功能需求

# 6.1 Windows客户端

## 设备发现

功能：

* 开启BLE扫描
* 显示附近设备
* 显示距离
* 显示设备名称

UI：

```
附近设备

📱 Pixel 9

距离:
2米


[连接]

```

---

## 文件发送

支持：

* 文件
* 文件夹
* 多文件

操作：

```
选择文件

↓

选择设备

↓

发送

```

---

## 文件接收

收到：

```
Pixel 9

发送:

photo.zip

大小:
2GB


[接受]

```

---

# 6.2 Android客户端

## 设备发现

显示：

```
附近电脑


💻 Desktop-PC


[连接]

```

---

## 扫码连接（增强功能）

支持：

Windows生成二维码：

```
AirDrop-X

Device:
PC-001

Session:
xxx

```

Android扫描：

```
扫码

↓

建立连接

```

---

## 文件选择

支持：

* 相册
* 文件管理器
* 分享菜单

---

# 7. 传输协议设计

## 7.1 Session协议

建立：

```
HELLO


{
version:1,
device_id:"",
public_key:""
}

```

---

## 7.2 文件协议

开始：

```
FILE_START

{
name:"",
size:"",
hash:""
}

```

数据：

```
FILE_CHUNK

{
index:1,
data:""
}

```

结束：

```
FILE_END

{
hash:""
}

```

---

# 8. 安全设计

## 8.1 加密

通信：

```
BLE:

ECDH


传输:

AES-256-GCM

```

---

## 8.2 防止中间人攻击

首次连接：

显示验证码：

```
839421
```

双方确认：

```
Windows:

839421


Android:

839421

```

---

# 9. 断点续传

支持：

* 网络断开
* 应用关闭
* 重新连接

保存：

```
transfer.db


file_id

chunk_status

hash

```

恢复：

```
已经完成:

0-500


继续:

501+

```

---

# 10. 技术方案

# Windows

推荐：

```
Rust

+

Tauri

+

tokio

+

quinn

+

windows-rs

```

模块：

```
windows-client

├── ble
│
├── wifi-direct
│
├── quic
│
├── crypto
│
└── ui

```

---

# Android

技术：

```
Kotlin


BluetoothLeScanner


WifiP2pManager


Cronet QUIC

```

模块：

```
android-client

├── ble

├── wifi

├── transfer

└── ui

```

---

# 11. MVP版本规划

## V0.1 基础版本

目标：

Windows ↔ Android 文件传输

实现：

* BLE发现
* WiFi Direct连接
* TCP传输
* 文件发送

---

## V0.2

增加：

* QUIC
* 加密
* 断点续传
* 文件夹传输

---

## V0.3

增加：

* QR扫码连接
* 历史设备
* 自动接受
* 分享菜单集成

---

## V1.0

完整AirDrop体验：

* 后台运行
* 附近设备自动出现
* 拖拽发送
* 秒级连接
* 百兆级速度

---

# 12. 开源设计

License:

推荐：

MIT

代码仓库：

```
airdrop-x

├── windows

├── android

├── protocol

├── docs

└── tests

```

---

# 13. 后续扩展

## 支持更多平台

未来：

* macOS
* Linux
* iOS

---

## 更多能力

* 剪贴板同步
* 图片快速预览
* 手机投屏
* 远程控制
* 通知同步

---

# 14. 成功标准

产品成功指标：

| 指标      | 目标   |
| ------- | ---- |
| 首次连接成功率 | 95%+ |
| 发现速度    | 3秒以内 |
| 大文件传输稳定 | 99%  |
| 用户无需学习  | 是    |
| 无需服务器   | 是    |

---

# 总结

AirDrop-X 使用：

```
BLE
 ↓
发现附近设备

Wi-Fi Direct
 ↓
建立高速无线链路

QUIC
 ↓
可靠高速文件传输
```

实现一个完全开源、无服务器、跨平台的 AirDrop 替代方案。

```
```
