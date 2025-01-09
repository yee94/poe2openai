use crate::types::{*, SettingsResponse};
use crate::error::PoeError;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::Client;
use serde_json::Value;
use std::pin::Pin;
use futures_util::Stream;
use reqwest::multipart::{Form, Part};
use reqwest::StatusCode;

const BASE_URL: &str = "https://api.poe.com/bot/";
const POE_GQL_URL: &str = "https://poe.com/api/gql_POST";
const POE_GQL_MODEL_HASH: &str = "b24b2f2f6da147b3345eec1a433ed17b6e1332df97dea47622868f41078a40cc";
const POE_GQL_UPLOAD_URL: &str = "https://poe.com/api/gql_upload_POST";


// curl 'https://poe.com/api/gql_upload_POST' \
//   -H 'accept: */*' \
//   -H 'accept-language: zh,en-US;q=0.9,en;q=0.8' \
//   -H 'content-type: multipart/form-data; boundary=----WebKitFormBoundary313d2Pbwo852ECfy' \
//   -H 'cookie: p-b=c0RU1x7kL7rVBcJRdL8YCg%3D%3D; _fbp=fb.1.1723446289172.310889772568305882; p-lat=tPuQQI7AmZzY7Ml6AJtOuxfXzyVoml413Eef4S8cLQ%3D%3D; _gcl_au=1.1.411570217.1735094430; cf_clearance=9d7TTxW_wNW65e8jhiMK2hdprddLRGalHxTKeXQvPTs-1735096580-1.2.1.1-0wId8jkjzCcyrtxs7stw1.ptVs46dnP.XEsMyfMQYfFC_uC2lNeod4fF5bBygtw8MwgFfJUADd7biFzzHO5B3FJwYDGJ8ub_hkblf8Pzv0gaphQiWJjcYUOCW4bG5arUF3rEq2dWlTHNBIP_8Xlkk2pxfv8mHv7NIVpbfIdbjnguRl.TIFeuGeu1iK77ZQ.UFpmrn2GU1wpJ1JFi4c6xFly7xdUxmH.UF8cLQZITS6YjzAhoSiarVzovKZ8.VF4e5YxdvOUMfT_GJMyl2DZg0ji8lvIillgtCVeeLCrcSTKfx5ghnzFes2uqjaeJuhLimjvkcF98hDWW3F6wcu7JE99GCdQPXbPYz_ynVPJ42mvYLwAF0gkFHt3mJKSXjGcxCjj35ADj1S2EBZ_S4nJzgg; __cf_bm=s0DPj4E6KxCge4ldIo17.ghnlrpPEJVvKVBkDB4pJXQ-1735096764-1.0.1.1-ZYEIYYoVQ5oe0qEVgN9lAFMvdGc9kss7oaa6m10S9CoHKUlZUHLqeHsVu.t7gup81cGiqFd3H4LRs.RFPnXB9w; OptanonConsent=isGpcEnabled=0&datestamp=Wed+Dec+25+2024+11%3A19%3A26+GMT%2B0800+(%E4%B8%AD%E5%9B%BD%E6%A0%87%E5%87%86%E6%97%B6%E9%97%B4)&version=202405.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&landingPath=NotLandingPage&groups=C0001%3A1%2CC0003%3A1%2CC0004%3A1&AwaitingReconsent=false&geolocation=HK%3B; OptanonAlertBoxClosed=2024-12-25T03:19:26.404Z' \
//   -H 'dnt: 1' \
//   -H 'origin: https://poe.com' \
//   -H 'poe-formkey: e648753bf2b77657eb58242f6b8b0e31' \
//   -H 'poe-queryname: sendMessageMutation' \
//   -H 'poe-tag-id: 9c087419200991b982cd827021247336' \
//   -H 'poe-tchannel: poe-chan53-8888-epvwaoixsrcsxrcglfuj' \
//   -H 'poegraphql: 0' \
//   -H 'priority: u=1, i' \
//   -H 'referer: https://poe.com/chat/2zn12dg7eyw6cs8rmpy' \
//   -H 'sec-ch-ua: "Chromium";v="131", "Not_A Brand";v="24"' \
//   -H 'sec-ch-ua-mobile: ?0' \
//   -H 'sec-ch-ua-platform: "macOS"' \
//   -H 'sec-fetch-dest: empty' \
//   -H 'sec-fetch-mode: cors' \
//   -H 'sec-fetch-site: same-origin' \
//   -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36' \
//   --data-raw $'------WebKitFormBoundary313d2Pbwo852ECfy\r\nContent-Disposition: form-data; name="queryInfo"\r\n\r\n{"queryName":"sendMessageMutation","variables":{"chatId":871160085,"bot":"claude_3_igloo","query":"阅读这个问题件","source":{"sourceType":"chat_input","chatInputMetadata":{"useVoiceRecord":false}},"clientNonce":"V4YEWDx94hx5AtDZ","sdid":"332d7088-e197-4227-937c-4bcc53aacfc7","attachments":["file0"],"existingMessageAttachmentsIds":[],"shouldFetchChat":false,"messagePointsDisplayPrice":380,"referencedMessageId":null},"extensions":{"hash":"68c788501b48e8662945256369c31fb92f6d04779514db35a929b36f054fe4f8"}}\r\n------WebKitFormBoundary313d2Pbwo852ECfy\r\nContent-Disposition: form-data; name="file0"; filename="pose.json"\r\nContent-Type: application/json\r\n\r\n\r\n------WebKitFormBoundary313d2Pbwo852ECfy--\r\n'

pub struct PoeClient {
    client: Client,
    bot_name: String,
    access_key: String,
}

impl PoeClient {
    pub fn new(bot_name: &str, access_key: &str) -> Self {
        Self {
            client: Client::new(),
            bot_name: bot_name.to_string(),
            access_key: access_key.to_string(),
        }
    }

    pub async fn stream_request(&self, request: QueryRequest) -> Result<Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>, PoeError> {
        let url = format!("{}{}", BASE_URL, self.bot_name);

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.access_key))
            .json(&request)
            .send()
            .await?;
    
        let mut static_buffer = String::new();
        let mut current_event: Option<EventType> = None;
        let mut is_collecting_data = false;
    
        let stream = response.bytes_stream().map(move |result| {
            result.map_err(PoeError::from).and_then(|chunk| {
                
                let chunk_str = String::from_utf8_lossy(&chunk);
                
                let mut events = Vec::new();
                
                // 將新的塊添加到靜態緩衝區
                static_buffer.push_str(&chunk_str);
                
                // 尋找完整的消息
                while let Some(newline_pos) = static_buffer.find('\n') {
                    let line = static_buffer[..newline_pos].trim().to_string();
                    static_buffer = static_buffer[newline_pos + 1..].to_string();             
                    
                    if line == ": ping" {
                        continue;
                    }
                    
                    if line.starts_with("event: ") {
                        let event_type = match line.trim_start_matches("event: ").trim() {
                            "text" => {
                                EventType::Text
                            },
                            "replace_response" => {
                                EventType::ReplaceResponse
                            },
                            "done" => {
                                EventType::Done
                            },
                            "error" => {
                                EventType::Error
                            },
                            _ => { 
                                continue;
                            }
                        };
                        current_event = Some(event_type);
                        is_collecting_data = false;
                        continue;
                    }
                    
                    if line.starts_with("data: ") {
                        let data = line.trim_start_matches("data: ").trim();
                        
                        if let Some(ref event_type) = current_event {
                            match event_type {
                                EventType::Text | EventType::ReplaceResponse => {
                                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                                        if let Some(text) = json.get("text").and_then(Value::as_str) {
                                            events.push(Ok(EventResponse {
                                                event: event_type.clone(),
                                                data: Some(PartialResponse {
                                                    text: text.to_string(),
                                                }),
                                                error: None,
                                            }));
                                        }
                                    } else {
                                        is_collecting_data = true;
                                    }
                                },
                                EventType::Done => {
                                    events.push(Ok(EventResponse {
                                        event: EventType::Done,
                                        data: None,
                                        error: None,
                                    }));
                                    current_event = None;
                                },
                                EventType::Error => {
                                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                                        let text = json.get("text")
                                            .and_then(Value::as_str)
                                            .unwrap_or("未知錯誤");
                                        let allow_retry = json.get("allow_retry")
                                            .and_then(Value::as_bool)
                                            .unwrap_or(false);
                                        
                                        events.push(Ok(EventResponse {
                                            event: EventType::Error,
                                            data: None,
                                            error: Some(ErrorResponse {
                                                text: text.to_string(),
                                                allow_retry,
                                            }),
                                        }));
                                    }
                                    current_event = None;
                                }
                            }
                        }
                    } else if is_collecting_data {
                        // 嘗試解析累積的 JSON
                        if let Some(ref event_type) = current_event {
                            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                                if let Some(text) = json.get("text").and_then(Value::as_str) {
                                    events.push(Ok(EventResponse {
                                        event: event_type.clone(),
                                        data: Some(PartialResponse {
                                            text: text.to_string(),
                                        }),
                                        error: None,
                                    }));
                                    is_collecting_data = false;
                                    current_event = None;
                                }
                            }
                        }
                    }
                }
                
                Ok(events)
            })
        })
        .flat_map(|result| {
            futures_util::stream::iter(match result {
                Ok(events) => events,
                Err(e) => {
                    vec![Err(e)]
                },
            })
        });

        Ok(Box::pin(stream))
    }

    pub async fn post_message_attachment(
        &self,
        message_id: &str,
        download_url: Option<&str>,
        download_filename: Option<&str>,
        file_data: Option<Vec<u8>>,
        filename: Option<&str>,
        content_type: Option<&str>,
        is_inline: bool,
    ) -> Result<AttachmentUploadResponse, PoeError> {
        let url = "https://www.quora.com/poe_api/file_attachment_3RD_PARTY_POST";
        let client = reqwest::Client::new();
        
        let mut request = client.post(url)
            .header("Authorization", format!("Bearer {}", self.access_key));

        println!("🔑 TOKEN: {}", self.access_key);

        if let Some(download_url) = download_url {
            let data = AttachmentRequest {
                message_id: message_id.to_string(),
                is_inline,
                download_url: Some(download_url.to_string()),
                download_filename: download_filename.map(|s| s.to_string()),
            };
            
            request = request.json(&data);
        } else if let (Some(file_data), Some(filename)) = (file_data, filename) {
            let file_part = match content_type {
                Some(ct) => Part::bytes(file_data)
                    .file_name(filename.to_string())
                    .mime_str(ct)?,
                None => Part::bytes(file_data)
                    .file_name(filename.to_string())
            };
            
            let form = Form::new()
                .text("message_id", message_id.to_string())
                .text("is_inline", is_inline.to_string())
                .part("file", file_part);
                
            request = request.multipart(form);
        } else {
            return Err(PoeError::InvalidParameter(
                "Must provide either download_url or file_data and filename".to_string()
            ));
        }

        let response = request.send().await?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(PoeError::AttachmentUpload(format!(
                "{} {}: {}", 
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                error_text
            )));
        }

        let upload_response = response.json::<AttachmentUploadResponse>().await?;
        Ok(upload_response)
    }

  
    // Fetch settings from the bot server
    pub async fn fetch_settings(&self) -> Result<SettingsResponse, PoeError> {
        let url = format!("{}fetch_settings/{}/{}", BASE_URL, self.bot_name, self.access_key);

        println!("🔑 URL: {}", url);
        
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.access_key))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(PoeError::BotError(format!(
                "Error fetching settings for bot {}: {}",
                self.bot_name,
                response.text().await?
            )));
        }

        let settings_response = response.json::<SettingsResponse>().await?;

        Ok(settings_response)
    }

    // Sync settings with the bot server
    pub async fn sync_bot_settings(
        &self,
        settings: Option<SettingsResponse>,
    ) -> Result<(), PoeError> {
        let url = format!("{}update_settings/{}/{}", BASE_URL, self.bot_name, self.access_key);
        println!("🔑 URL: {}", url);
        
        let response = if let Some(settings) = settings {

        println!("🔑 SETTINGS: {:?}", settings);

            self.client.post(&url)
                .json(&settings)
                .send()
                .await?
        } else {
            self.client.post(&url)
                .header("Authorization", format!("Bearer {}", self.access_key))
                .send()
                .await?
        };

        if response.status() != StatusCode::OK {
            return Err(PoeError::BotError(format!(
                "Error syncing settings for bot {}: {}",
                self.bot_name,
                response.text().await?
            )));
        }

        Ok(())
    }

    pub fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", self.access_key)).unwrap());
        headers.insert("Accept-Language", HeaderValue::from_static("zh,en-US;q=0.9,en;q=0.8"));
        headers.insert("Origin", HeaderValue::from_static("https://poe.com"));
        headers.insert("Referer", HeaderValue::from_static("https://poe.com"));
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
        headers
    }

    pub async fn gql_upload_post(
        &self,
        query_info: QueryRequest,
        file_data: Option<Vec<u8>>,
        filename: &str,
    ) -> Result<Value, PoeError> {
        let url = POE_GQL_UPLOAD_URL;

        // Serialize the QueryRequest to JSON
        let query_info_json = serde_json::to_string(&query_info)
            .map_err(|e| PoeError::JsonParseFailed(e))?;

        // Create the multipart form
        let mut form = Form::new()
            .text("queryInfo", query_info_json);

        if let Some(data) = file_data {
            let file_part = Part::bytes(data)
                .file_name(filename.to_string())
                .mime_str("application/json")?; // Set the appropriate content type

            form = form.part("file0", file_part);
        }

        // Send the request
        let response = self.client.post(url)
            .headers(self.headers()) // Use the headers method to include authorization
            .multipart(form)
            .send()
            .await?;

        // Check the response status
        if !response.status().is_success() {
            return Err(PoeError::BotError(format!(
                "Error uploading file: {}",
                response.text().await?
            )));
        }

        // Parse the JSON response
        let json_response = response.json::<Value>().await?;
        Ok(json_response)
    }
}

pub async fn get_model_list(language_code: Option<&str>) -> Result<ModelListResponse, PoeError> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| PoeError::BotError(e.to_string()))?;

    let payload = serde_json::json!({
        "queryName": "ExploreBotsListPaginationQuery",
        "variables": {
            "categoryName": "defaultCategory",
            "count": 150
        },
        "extensions": {
            "hash": POE_GQL_MODEL_HASH
        }
    });

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert("Accept-Language", HeaderValue::from_static("zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7"));
    headers.insert("Origin", HeaderValue::from_static("https://poe.com"));
    headers.insert("Referer", HeaderValue::from_static("https://poe.com"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert("poegraphql", HeaderValue::from_static("1"));

    if let Some(code) = language_code {
        let cookie_value = format!("Poe-Language-Code={}; p-b=1", code);
        headers.insert(COOKIE, HeaderValue::from_str(&cookie_value)
            .map_err(|e| PoeError::BotError(e.to_string()))?);
    }

    let response = client.post(POE_GQL_URL)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| PoeError::RequestFailed(e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "無法讀取回應內容".to_string());
        return Err(PoeError::BotError(format!("API 回應錯誤 - 狀態碼: {}, 內容: {}", status, text)));
    }

    let json_value = response.text().await
        .map_err(|e| PoeError::RequestFailed(e))?;
    
    let data: Value = serde_json::from_str(&json_value)
        .map_err(|e| PoeError::JsonParseFailed(e))?;
    
    let mut model_list = Vec::with_capacity(150);
    if let Some(edges) = data["data"]["exploreBotsConnection"]["edges"].as_array() {
        for edge in edges {
            if let Some(handle) = edge["node"]["handle"].as_str() {
                model_list.push(ModelInfo {
                    id: handle.to_string(),
                    object: "model".to_string(),
                    created: 0,
                    owned_by: "poe".to_string(),
                });
            }
        }
    } else {
        return Err(PoeError::BotError("無法從回應中取得模型列表".to_string()));
    }

    if model_list.is_empty() {
        return Err(PoeError::BotError("取得的模型列表為空".to_string()));
    }

    Ok(ModelListResponse { data: model_list })
}