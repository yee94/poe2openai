use salvo::prelude::*;
mod types;
mod handlers;
mod poe_client;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .push(Router::with_path("models").get(handlers::get_models))
        .push(Router::with_path("chat/completions").post(handlers::chat_completions))
        .push(Router::with_path("v1/models").get(handlers::get_models))
        .push(Router::with_path("v1/chat/completions").post(handlers::chat_completions));

    let acceptor = TcpListener::new("0.0.0.0:7070").bind().await;
    Server::new(acceptor).serve(router).await;
}