use futures_util::future;
use futures_util::stream::{self, Stream, StreamExt};
use nanoid::nanoid;
use poe_api_process::{EventResponse, EventType, PoeError};
use salvo::http::header;
use salvo::prelude::*;
use serde_json::json;
use std::pin::Pin;
use std::time::Instant;
use tracing::{debug, error, info};
use chrono::Utc;

use crate::poe_client::{PoeClientWrapper, create_query_request};
use crate::types::*;
use crate::utils::{format_bytes_length, format_duration, truncate_text};

#[handler]
pub async fn chat_completions(req: &mut Request, res: &mut Response) {
    let start_time = Instant::now();
    info!("📝 收到新的聊天完成請求");
    
    let access_key = match req.headers().get("Authorization") {
        Some(auth) => {
            let auth_str = auth.to_str().unwrap_or("");
            if auth_str.starts_with("Bearer ") {
                debug!("🔑 驗證令牌長度: {}", auth_str[7..].len());
                auth_str[7..].to_string()
            } else {
                error!("❌ 無效的授權格式");
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(json!({ "error": "無效的 Authorization" })));
                return;
            }
        },
        None => {
            error!("❌ 缺少授權標頭");
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(json!({ "error": "缺少 Authorization" })));
            return;
        }
    };

    let chat_request = match req.parse_json::<ChatCompletionRequest>().await {
        Ok(req) => {
            debug!("📊 請求解析成功 | 模型: {} | 訊息數量: {} | 是否串流: {:?}", 
                req.model, 
                req.messages.len(),
                req.stream
            );
            req
        },
        Err(e) => {
            error!("❌ 請求解析失敗: {}", e);
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({ "error": e.to_string() })));
            return;
        }
    };

    info!("🤖 使用模型: {}", chat_request.model);

    let client = PoeClientWrapper::new(&chat_request.model, &access_key);
    let query_request = create_query_request(chat_request.messages, chat_request.temperature);
    let stream = chat_request.stream.unwrap_or(false);
    
    debug!("🔄 請求模式: {}", if stream { "串流" } else { "非串流" });

    match client.stream_request(query_request).await {
        Ok(event_stream) => {
            if stream {
                handle_stream_response(res, event_stream, &chat_request.model).await;
            } else {
                handle_non_stream_response(res, event_stream, &chat_request.model).await;
            }
        },
        Err(e) => {
            error!("❌ 建立串流請求失敗: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({ "error": e.to_string() })));
        }
    }

    let duration = start_time.elapsed();
    info!("✅ 請求處理完成 | 耗時: {}", format_duration(duration));
}

fn convert_poe_error_to_openai(error: &poe_api_process::types::ErrorResponse) -> (StatusCode, OpenAIErrorResponse) {
    debug!("🔄 轉換錯誤響應 | 錯誤文本: {}", error.text);
    
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

    debug!("📋 錯誤轉換結果 | 狀態碼: {} | 錯誤類型: {}", status.as_u16(), error_type);

    (status, OpenAIErrorResponse {
        error: OpenAIError {
            message: error.text.clone(),
            r#type: error_type.to_string(),
            code: code.to_string(),
            param: None,
        }
    })
}

async fn handle_stream_response(
    res: &mut Response,
    mut event_stream: Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>,
    model: &str
) {
    let start_time = Instant::now();
    let id = nanoid!(10);
    let created = Utc::now().timestamp();
    let model = model.to_string(); // 轉換為擁有的 String
    
    info!("🌊 開始處理串流響應 | ID: {} | 模型: {}", id, model);

    res.headers_mut().insert(header::CONTENT_TYPE, "text/event-stream".parse().unwrap());
    res.headers_mut().insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    res.headers_mut().insert(header::CONNECTION, "keep-alive".parse().unwrap());

    let mut replace_response = false;
    let mut full_content = String::new();
    let mut first_two_events = Vec::new();

    debug!("🔍 檢查初始事件");
    for _ in 0..2 {
        if let Some(Ok(event)) = event_stream.next().await {
            debug!("📥 收到初始事件: {:?}", event.event);
            first_two_events.push(event);
        }
    }

    for event in first_two_events {
        match event.event {
            EventType::ReplaceResponse => {
                debug!("🔄 檢測到 ReplaceResponse 模式");
                replace_response = true;
                if let Some(data) = event.data {
                    debug!("📝 初始內容長度: {}", format_bytes_length(data.text.len()));
                    full_content = data.text;
                }
            },
            EventType::Text => {
                if let Some(data) = event.data {
                    debug!("📝 收到文本: {}", truncate_text(&data.text, 50));
                    if !replace_response {
                        full_content.push_str(&data.text);
                    }
                }
            },
            EventType::Error => {
                if !replace_response {
                    if let Some(error) = event.error {
                        error!("❌ 串流處理錯誤: {}", error.text);
                        let (status, error_response) = convert_poe_error_to_openai(&error);
                        res.status_code(status);
                        res.render(Json(error_response));
                        return;
                    }
                }
            },
            EventType::Done => {
                debug!("✅ 初始事件處理完成");
                break;
            }
        }
    }

    let id_for_log = id.clone(); // 為最後的日誌克隆一個副本

    if replace_response {
        debug!("🔄 使用 ReplaceResponse 處理模式");
        let processed_stream = {
            let id = id.clone(); // 為閉包克隆一個副本
            let model = model.clone(); // 為閉包克隆一個副本
            
            stream::once(async move {
                let content = handle_replace_response(event_stream).await;
                debug!("📤 處理完成 | 內容長度: {}", format_bytes_length(content.len()));
                
                let content_chunk = create_stream_chunk(&id, created, &model, &content, None);
                let content_json = serde_json::to_string(&content_chunk).unwrap();
                let content_message = format!("data: {}\n\n", content_json);
                
                let final_chunk = create_stream_chunk(&id, created, &model, "", Some("stop".to_string()));
                let final_json = serde_json::to_string(&final_chunk).unwrap();
                let final_message = format!("{}data: {}\n\ndata: [DONE]\n\n", 
                    content_message, final_json);
                
                Ok::<_, std::convert::Infallible>(final_message)
            })
        };

        res.stream(processed_stream);
    } else {
        debug!("🔄 使用標準串流處理模式");
        let initial_chunk = create_stream_chunk(&id, created, &model, &full_content, None);
        let initial_chunk_json = serde_json::to_string(&initial_chunk).unwrap();
        let initial_message = format!("data: {}\n\n", initial_chunk_json);

        let processed_stream = {
            let id = id.clone(); // 為閉包克隆一個副本
            let model = model.clone(); // 為閉包克隆一個副本
            
            stream::once(future::ready(Ok::<_, std::convert::Infallible>(initial_message)))
                .chain(stream::unfold(
                    (event_stream, false),
                    move |(mut event_stream, mut is_done)| {
                        let id = id.clone();
                        let model = model.clone();
                        
                        async move {
                            if is_done {
                                debug!("✅ 串流處理完成");
                                return None;
                            }
                            match event_stream.next().await {
                                Some(Ok(event)) => {
                                    match event.event {
                                        EventType::Text => {
                                            if let Some(data) = event.data {
                                                debug!("📝 處理文本片段: {}", truncate_text(&data.text, 50));
                                                let chunk = create_stream_chunk(&id, created, &model, &data.text, None);
                                                let chunk_json = serde_json::to_string(&chunk).unwrap();
                                                Some((Ok(format!("data: {}\n\n", chunk_json)), (event_stream, is_done)))
                                            } else {
                                                Some((Ok(String::new()), (event_stream, is_done)))
                                            }
                                        },
                                        EventType::Error => {
                                            if let Some(error) = event.error {
                                                error!("❌ 串流處理錯誤: {}", error.text);
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
                                            debug!("✅ 串流完成");
                                            is_done = true;
                                            let final_chunk = create_stream_chunk(&id, created, &model, "", Some("stop".to_string()));
                                            let final_chunk_json = serde_json::to_string(&final_chunk).unwrap();
                                            Some((Ok(format!("data: {}\n\ndata: [DONE]\n\n", final_chunk_json)), (event_stream, is_done)))
                                        },
                                        _ => {
                                            debug!("⏭️ 忽略其他事件類型");
                                            Some((Ok(String::new()), (event_stream, is_done)))
                                        },
                                    }
                                },
                                _ => None,
                            }
                        }
                    },
                ))
        };

        res.stream(processed_stream);
    }

    let duration = start_time.elapsed();
    info!("✅ 串流響應處理完成 | ID: {} | 耗時: {}", id_for_log, format_duration(duration));
}

async fn handle_non_stream_response(
    res: &mut Response,
    mut event_stream: Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>,
    model: &str
) {
    let start_time = Instant::now();
    let id = nanoid!(10);
    
    info!("📦 開始處理非串流響應 | ID: {} | 模型: {}", id, model);

    let mut replace_response = false;
    let mut full_content = String::new();
    let mut first_two_events = Vec::new();

    debug!("🔍 檢查初始事件");
    for _ in 0..2 {
        if let Some(Ok(event)) = event_stream.next().await {
            debug!("📥 收到初始事件: {:?}", event.event);
            first_two_events.push(event);
        }
    }

    for event in first_two_events {
        match event.event {
            EventType::ReplaceResponse => {
                debug!("🔄 檢測到 ReplaceResponse 模式");
                replace_response = true;
                if let Some(data) = event.data {
                    debug!("📝 初始內容長度: {}", format_bytes_length(data.text.len()));
                    full_content = data.text;
                }
            },
            EventType::Text => {
                if let Some(data) = event.data {
                    debug!("📝 收到文本: {}", truncate_text(&data.text, 50));
                    if !replace_response {
                        full_content.push_str(&data.text);
                    }
                }
            },
            EventType::Error => {
                if let Some(error) = event.error {
                    error!("❌ 處理錯誤: {}", error.text);
                    let (status, error_response) = convert_poe_error_to_openai(&error);
                    res.status_code(status);
                    res.render(Json(error_response));
                    return;
                }
            },
            EventType::Done => {
                debug!("✅ 初始事件處理完成");
                break;
            }
        }
    }

    if replace_response {
        debug!("🔄 使用 ReplaceResponse 處理模式");
        let content = handle_replace_response(event_stream).await;
        debug!("📤 最終內容長度: {}", format_bytes_length(content.len()));

        let response = ChatCompletionResponse {
            id: format!("chatcmpl-{}", nanoid!(10)),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![CompletionChoice {
                index: 0,
                message: CompletionMessage {
                    role: "assistant".to_string(),
                    content,
                    refusal: None,
                },
                logprobs: None,
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        res.render(Json(response));
    } else {
        debug!("🔄 使用標準非串流處理模式");
        let mut response_content = full_content;

        while let Some(Ok(event)) = event_stream.next().await {
            match event.event {
                EventType::Text => {
                    if let Some(data) = event.data {
                        debug!("📝 處理文本片段: {}", truncate_text(&data.text, 50));
                        response_content.push_str(&data.text);
                    }
                },
                EventType::Error => {
                    if let Some(error) = event.error {
                        error!("❌ 處理錯誤: {}", error.text);
                        let (status, error_response) = convert_poe_error_to_openai(&error);
                        res.status_code(status);
                        res.render(Json(error_response));
                        return;
                    }
                },
                EventType::Done => {
                    debug!("✅ 回應收集完成");
                    break;
                },
                _ => {
                    debug!("⏭️ 忽略其他事件類型");
                }
            }
        }

        debug!("📤 準備發送回應 | 內容長度: {}", format_bytes_length(response_content.len()));
        let response = ChatCompletionResponse {
            id: format!("chatcmpl-{}", id),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![CompletionChoice {
                index: 0,
                message: CompletionMessage {
                    role: "assistant".to_string(),
                    content: response_content,
                    refusal: None,
                },
                logprobs: None,
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        res.render(Json(response));
    }

    let duration = start_time.elapsed();
    info!("✅ 非串流響應處理完成 | ID: {} | 耗時: {}", id, format_duration(duration));
}

async fn handle_replace_response(
    mut event_stream: Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>,
) -> String {
    let start_time = Instant::now();
    debug!("🔄 開始處理 ReplaceResponse");
    
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;
    
    let (tx, mut rx) = mpsc::channel(1);
    let last_content = Arc::new(Mutex::new(String::new()));
    let last_content_clone = Arc::clone(&last_content);
    
    tokio::spawn(async move {
        debug!("🏃 啟動背景事件收集任務");
        let mut done_received = false;
        
        while !done_received {
            match event_stream.next().await {
                Some(Ok(event)) => {
                    match event.event {
                        EventType::ReplaceResponse => {
                            if let Some(data) = event.data {
                                debug!("📝 更新內容 | 長度: {}", format_bytes_length(data.text.len()));
                                *last_content_clone.lock().unwrap() = data.text;
                            }
                        },
                        EventType::Done => {
                            debug!("✅ 收到完成信號");
                            done_received = true;
                            let _ = tx.send(()).await;
                        },
                        _ => {
                            debug!("⏭️ 忽略其他事件類型");
                        }
                    }
                },
                Some(Err(e)) => {
                    error!("❌ 事件處理錯誤: {:?}", e);
                },
                None => {
                    debug!("⚠️ 事件流結束但未收到完成信號");
                    break;
                }
            }
        }
        debug!("👋 背景任務結束");
    });

    let _ = rx.recv().await;
    let final_content = {
        let content = last_content.lock().unwrap();
        content.clone()
    };

    let duration = start_time.elapsed();
    debug!("✅ ReplaceResponse 處理完成 | 內容長度: {} | 耗時: {}", 
        format_bytes_length(final_content.len()),
        format_duration(duration)
    );
    
    final_content
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

    debug!("🔧 創建串流片段 | ID: {} | 內容長度: {}", 
        id,
        if let Some(content) = &delta.content {
            format_bytes_length(content.len())
        } else {
            "0 B".to_string()
        }
    );

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