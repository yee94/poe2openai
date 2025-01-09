FROM debian:bookworm-slim

# 設定執行時的環境變數
ENV HOST=0.0.0.0 \
    PORT=8080 \
    ADMIN_USERNAME=admin \
    ADMIN_PASSWORD=123456 \
    MAX_REQUEST_SIZE=1073741824 \
    LOG_LEVEL=info \
    RUST_BACKTRACE=1 \
    TZ=Asia/Taipei

# 安裝執行時期依賴
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone

# 建立非 root 使用者
RUN groupadd -r poe && useradd -r -g poe poe

# 建立應用程式目錄
WORKDIR /app

# 從建構階段複製編譯好的二進制檔案和資源文件
COPY ./target/release/poe2openai /app/
COPY ./templates /app/templates
COPY ./static /app/static

# 設定檔案權限
RUN chown -R poe:poe /app

# 切換到非 root 使用者
USER poe

# 設定容器啟動指令
ENTRYPOINT ["/app/poe2openai"]

# 暴露端口
EXPOSE ${PORT}

# 設定標籤
LABEL maintainer="Jerome Leong <jeromeleong1998@gmail.com>" \
    description="Poe API to OpenAI API 轉換服務" \
    version="0.2.1"