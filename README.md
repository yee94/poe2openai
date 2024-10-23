# 🔄 POE to OpenAI API 轉換器

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

將 POE API 轉換為 OpenAI API 格式的代理服務器。讓您能夠使用 OpenAI API 的客戶端來無縫接入 POE 的服務。

## 📑 目錄
- [主要特點](#主要特點)
- [安裝指南](#安裝指南)
- [快速開始](#快速開始)
- [API 文檔](#api-文檔)
- [配置說明](#配置說明)
- [常見問題](#常見問題)
- [貢獻指南](#貢獻指南)
- [授權協議](#授權協議)

## ✨ 主要特點
- 🔄 完整支援 OpenAI API 格式
- 🚀 高效能 Rust 實現
- 💬 支援串流（Stream）輸出
- 🔑 POE API 金鑰認證
- 🌐 對 POE API 的 Event 進行完整處理

## 🔧 安裝指南

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

1. 啟動服務器：
```bash
poe2openai
```

2. 服務器默認在 `http://localhost:7070` 啟動

3. 使用方式示例：
```bash
curl http://localhost:7070/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-poe-token" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "你好"}],
    "stream": true
  }'
```

## 📖 API 文檔

### 支援的端點

- `GET /v1/models` - 獲取可用模型列表
- `POST /v1/chat/completions` - 與Poe模型聊天
- `GET /models` - 獲取可用模型列表（相容端點）
- `POST /chat/completions` - 與Poe模型聊天（相容端點）

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
  "model": "claude-2-100k",
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

- `PORT` - 服務器端口（默認：7070）
- `HOST` - 服務器主機（默認：0.0.0.0）

## ❓ 常見問題

### Q: 為什麼會收到認證錯誤？
A: 確保在請求頭中正確設置了 `Authorization: Bearer your-poe-token`

### Q: 支援哪些模型？
A: 支援所有 POE 平台上可用的模型，可通過 `/v1/models` 端點查詢官方模型

## 🤝 貢獻指南

歡迎所有形式的貢獻！

## 📄 授權協議
使用 [MIT LICENSE](LICENSE)