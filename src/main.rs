use salvo::prelude::*;
use tracing::{info, debug};
use std::env;
mod types;
mod handlers;
mod poe_client;
mod utils;

fn get_env_or_default(key: &str, default: &str) -> String {
    let value = env::var(key).unwrap_or_else(|_| default.to_string());
    debug!("🔧 環境變數 {} = {}", key, value);
    value
}

fn setup_logging(log_level: &str) {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(log_level)
        .init();

    info!("🚀 日誌系統初始化完成，日誌級別: {}", log_level);
}

#[tokio::main]
async fn main() {
    let log_level = get_env_or_default("LOG_LEVEL", "debug");
    setup_logging(&log_level);
    
    let host = get_env_or_default("HOST", "0.0.0.0");
    let port = get_env_or_default("PORT", "8080");
    let bind_address = format!("{}:{}", host, port);

    info!("🌟 正在啟動 Poe API To OpenAI API 服務...");
    debug!("📍 服務綁定地址: {}", bind_address);

    let router = Router::new()
        .push(Router::with_path("models").get(handlers::get_models))
        .push(Router::with_path("chat/completions").post(handlers::chat_completions))
        .push(Router::with_path("v1/models").get(handlers::get_models))
        .push(Router::with_path("v1/chat/completions").post(handlers::chat_completions));

    info!("🛣️  API 路由配置完成");
    
    let acceptor = TcpListener::new(&bind_address).bind().await;
    info!("🎯 服務已啟動並監聽於 {}", bind_address);
    
    Server::new(acceptor).serve(router).await;
}