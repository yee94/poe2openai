use futures_util::future;
use nanoid::nanoid;
use poe_api_process::{EventResponse, EventType, PoeError};
use tracing::{error, info};
use std::pin::Pin;
use salvo::http::header;
use salvo::prelude::*;
use chrono::Utc;
use futures_util::stream::{self, Stream, StreamExt};
use serde_json::json;
use crate::types::*;
use crate::poe_client::{PoeClientWrapper, create_query_request};

#[handler]
pub async fn chat_completions(req: &mut Request, res: &mut Response) {
    info!("收到模型聊天請求");
    let access_key = match req.headers().get("Authorization") {
        Some(auth) => {
            let auth_str = auth.to_str().unwrap_or("");
            if auth_str.starts_with("Bearer ") {
                auth_str[7..].to_string()
            } else {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(json!({ "error": "無效的 Authorization" })));
                return;
            }
        },
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(json!({ "error": "缺少 Authorization" })));
            return;
        }
    };

    let chat_request: ChatCompletionRequest = match req.parse_json().await {
        Ok(req) => {
            info!("成功解析請求內容");
            req
        },
        Err(e) => {
            error!("解析請求失敗: {}", e);
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({ "error": e.to_string() })));
            return;
        }
    };

    info!("使用模型: {}", chat_request.model);

    let client = PoeClientWrapper::new(&chat_request.model, &access_key);

    let query_request = create_query_request(chat_request.messages, chat_request.temperature);

    let stream = chat_request.stream.unwrap_or(false);

    match client.stream_request(query_request).await {
        Ok(event_stream) => {
            if stream {
                handle_stream_response(res, event_stream, &chat_request.model).await;
            } else {
                handle_non_stream_response(res, event_stream, &chat_request.model).await;
            }
        },
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({ "error": e.to_string() })));
        }
    }
}

fn convert_poe_error_to_openai(error: &poe_api_process::types::ErrorResponse) -> (StatusCode, OpenAIErrorResponse) {
    // 根據錯誤訊息判斷類型
    let (status, error_type, code) = if error.text.contains("Internal server error") {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "internal_error"
        )
    } else if error.text.contains("rate limit") {
        (
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limit_exceeded",
            "rate_limit_exceeded"
        )
    } else if error.text.contains("Invalid token") || error.text.contains("Unauthorized") {
        (
            StatusCode::UNAUTHORIZED,
            "invalid_auth",
            "invalid_api_key"
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "bad_request"
        )
    };

    (status, OpenAIErrorResponse {
        error: OpenAIError {
            message: error.text.clone(),
            r#type: error_type.to_string(),
            code: code.to_string(),
            param: None,
        }
    })
}

async fn handle_stream_response(res: &mut Response, mut event_stream: Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>, model: &str) {
    let id = nanoid!(10);
    let created = Utc::now().timestamp();
    let model = model.to_string();

    // 設置流式響應的標頭
    res.headers_mut().insert(header::CONTENT_TYPE, "text/event-stream".parse().unwrap());
    res.headers_mut().insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    res.headers_mut().insert(header::CONNECTION, "keep-alive".parse().unwrap());

    // 檢查前兩個事件
    let mut replace_response = false;
    let mut full_content = String::new();

    for _ in 0..2 {
        if let Some(Ok(event)) = event_stream.next().await {
            match event.event {
                EventType::ReplaceResponse => {
                    replace_response = true;
                    if let Some(data) = event.data {
                        full_content = data.text;
                    }
                    break;
                },
                EventType::Text => {
                    if let Some(data) = event.data {
                        full_content.push_str(&data.text);
                    }
                },
                EventType::Error => {
                    if let Some(error) = event.error {
                        let (status, error_response) = convert_poe_error_to_openai(&error);
                        res.status_code(status);
                        res.render(Json(error_response));
                        return;
                    }
                },
                EventType::Done => break,
            }
        } else {
            break;
        }
    }

    if replace_response {
        // 如果遇到 ReplaceResponse，切換到非流式處理
        handle_non_stream_response(res, event_stream, &model).await;
    } else {
        // 繼續流式處理
        let initial_chunk = create_stream_chunk(&id, created, &model, &full_content, None);
        let initial_chunk_json = serde_json::to_string(&initial_chunk).unwrap();
        let initial_message = format!("data: {}\n\n", initial_chunk_json);

        let id = id.clone();

        let processed_stream = stream::once(future::ready(Ok::<_, std::convert::Infallible>(initial_message)))
            .chain(stream::unfold(
                (event_stream, false),
                move |(mut event_stream, mut is_done)| {
                    let model = model.clone();
                    let id = id.clone(); // 在每次迭代時克隆 id
                    
                    async move {
                        if is_done {
                            return None;
                        }
                        match event_stream.next().await {
                            Some(Ok(event)) => {
                                match event.event {
                                    EventType::Text => {
                                        if let Some(data) = event.data {
                                            let chunk = create_stream_chunk(&id, created, &model, &data.text, None);
                                            let chunk_json = serde_json::to_string(&chunk).unwrap();
                                            Some((Ok(format!("data: {}\n\n", chunk_json)), (event_stream, is_done)))
                                        } else {
                                            Some((Ok(String::new()), (event_stream, is_done)))
                                        }
                                    },
                                    EventType::Error => {
                                        if let Some(error) = event.error {
                                            // 在串流模式下，將錯誤轉換為 SSE 格式
                                            let error_chunk = json!({
                                                "error": {
                                                    "message": error.text,
                                                    "type": "stream_error",
                                                    "code": "stream_error"
                                                }
                                            });
                                            let error_message = format!("data: {}\n\ndata: [DONE]\n\n", 
                                                serde_json::to_string(&error_chunk).unwrap());
                                            Some((Ok(error_message), (event_stream, true)))
                                        } else {
                                            Some((Ok(String::new()), (event_stream, true)))
                                        }
                                    },
                                    EventType::Done => {
                                        is_done = true;
                                        let final_chunk = create_stream_chunk(&id, created, &model, "", Some("stop".to_string()));
                                        let final_chunk_json = serde_json::to_string(&final_chunk).unwrap();
                                        Some((Ok(format!("data: {}\n\ndata: [DONE]\n\n", final_chunk_json)), (event_stream, is_done)))
                                    },
                                    _ => Some((Ok(String::new()), (event_stream, is_done))),
                                }
                            },
                            _ => None,
                        }
                    }
                },
            ));

        res.stream(processed_stream);
    }
}

async fn handle_non_stream_response(res: &mut Response, mut event_stream: Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>, model: &str) {
    let mut full_content = String::new();

    while let Some(Ok(event)) = event_stream.next().await {
        match event.event {
            EventType::Text => {
                if let Some(partial) = event.data {
                    full_content.push_str(&partial.text);  // 累積內容
                }
            },
            EventType::ReplaceResponse => {
                if let Some(partial) = event.data {
                    full_content = partial.text;  // 替換內容
                }
            },
            EventType::Error => {
                if let Some(error) = event.error {
                    let (status, error_response) = convert_poe_error_to_openai(&error);
                    res.status_code(status);
                    res.render(Json(error_response));
                    return;
                }
                break;
            },
            EventType::Done => break
        }
    }

    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", nanoid!(10)),
        object: "chat.completion".to_string(),
        created: Utc::now().timestamp(),
        model: model.to_string(),
        choices: vec![CompletionChoice {
            index: 0,
            message: CompletionMessage {
                role: "assistant".to_string(),
                content: full_content,
                refusal: None,
            },
            logprobs: None,
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    res.render(Json(response));
}

fn create_stream_chunk(id: &str, created: i64, model: &str, content: &str, finish_reason: Option<String>) -> ChatCompletionChunk {
    let mut delta = Delta {
        role: None,
        content: None,
        refusal: None,
    };

    if content.is_empty() && finish_reason.is_none() {
        delta.role = Some("assistant".to_string());
    } else {
        delta.content = Some(content.to_string());
    }

    ChatCompletionChunk {
        id: format!("chatcmpl-{}", id),
        object: "chat.completion.chunk".to_string(),
        created,
        model: model.to_string(),
        choices: vec![Choice {
            index: 0,
            delta,
            finish_reason,
        }],
    }
}