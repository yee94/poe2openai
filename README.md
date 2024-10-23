# 🔄 POE to OpenAI API 轉換器

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docker Size](https://img.shields.io/docker/image-size/jeromeleong/poe2openai/0.1.0
)](https://hub.docker.com/r/jeromeleong/poe2openai)
[![Docker Pulls](https://img.shields.io/docker/pulls/jeromeleong/poe2openai)](https://hub.docker.com/r/jeromeleong/poe2openai)

將 POE API 轉換為 OpenAI API 格式的代理服務器。讓您能夠使用 OpenAI API 的客戶端來無縫接入 POE 的服務。

## 📑 目錄
- [主要特點](#-主要特點)
- [安裝指南](#-安裝指南)
- [快速開始](#-快速開始)
- [API 文檔](#-api-文檔)
- [配置說明](#️-配置說明)
- [常見問題](#-常見問題)
- [貢獻指南](#-貢獻指南)
- [授權協議](#-授權協議)

## ✨ 主要特點
- 🔄 完整支援 OpenAI API 格式
- 🚀 高效能 Rust 實現
- 💬 支援串流（Stream）輸出
- 🔑 POE API 金鑰認證
- 🌐 對 POE API 的 Event 進行完整處理

## 🔧 安裝指南

### 使用 Docker（推薦）

```bash
# 拉取映像
docker pull jeromeleong/poe2openai:latest

# 運行容器
docker run -d -p 8080:8080 jeromeleong/poe2openai:latest
```

### 使用 Docker Compose

```yaml
version: '3.8'
services:
  poe2openai:
    image: jeromeleong/poe2openai:latest
    ports:
      - "8080:8080"
    environment:
      - PORT=8080
      - LOG_LEVEL=info
```

### 從源碼編譯

```bash
# 克隆專案
git clone https://github.com/jeromeleong/poe2openai
cd poe2openai

# 編譯
cargo build --release

# 運行
./target/release/poe2openai
```

## 🚀 快速開始

1. 使用 Docker 啟動服務：
```bash
docker run -d -p 8080:8080 jeromeleong/poe2openai:latest
```

2. 服務器默認在 `http://localhost:8080` 啟動

3. 使用方式示例：
```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-poe-token" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "你好"}],
    "stream": false
  }'
```

## 📖 API 文檔

### 支援的端點

- `GET /v1/models` - 獲取可用模型列表
- `POST /v1/chat/completions` - 與 POE 模型聊天
- `GET /models` - 獲取可用模型列表（相容端點）
- `POST /chat/completions` - 與 POE 模型聊天（相容端點）

### 請求格式
```json
{
  "model": "string",
  "messages": [
    {
      "role": "user",
      "content": "string"
    }
  ],
  "temperature": 0.7,
  "stream": false
}
```

### 響應格式

```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1677858242,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "回應內容"
      },
      "finish_reason": "stop"
    }
  ]
}
```

## ⚙️ 配置說明

服務器配置通過環境變量進行：

- `PORT` - 服務器端口（默認：8080）
- `HOST` - 服務器主機（默認：0.0.0.0）
- `LOG_LEVEL` - 日誌級別（默認：info）

## ❓ 常見問題

### Q: 為什麼會收到認證錯誤？
A: 確保在請求頭中正確設置了 `Authorization: Bearer your-poe-token`

### Q: 支援哪些模型？
A: 支援所有 POE 平台上可用的模型，可通過 `/v1/models` 端點查詢

### Q: 如何修改服務器端口？
A: 可以通過設置環境變量 `PORT` 來修改，例如：
```bash
docker run -d -e PORT=3000 -p 3000:3000 jeromeleong/poe2openai:latest
```

## 🤝 貢獻指南

歡迎所有形式的貢獻！請遵循以下步驟：

1. Fork 本專案
2. 創建新的功能分支
3. 提交您的更改
4. 提出 Pull Request

## 📄 授權協議

本專案使用 [MIT 授權協議](LICENSE)。