use salvo::prelude::*;
use tracing::info;
use std::env;
mod types;
mod handlers;
mod poe_client;

fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() {
    // 設定日誌級別
    let log_level = get_env_or_default("LOG_LEVEL", "info");
    
    // 設定服務監聽位址
    let host = get_env_or_default("HOST", "0.0.0.0");
    let port = get_env_or_default("PORT", "8080");
    let bind_address = format!("{}:{}", host, port);

    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(log_level)
        .init();

    info!("正在啟動 Poe API To Openai API 服務...");

    let router = Router::new()
        .push(Router::with_path("models").get(handlers::get_models))
        .push(Router::with_path("chat/completions").post(handlers::chat_completions))
        .push(Router::with_path("v1/models").get(handlers::get_models))
        .push(Router::with_path("v1/chat/completions").post(handlers::chat_completions));

    let acceptor = TcpListener::new(&bind_address).bind().await;
    
    Server::new(acceptor).serve(router).await;
}