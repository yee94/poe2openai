use salvo::prelude::*;
use poe_api_process::{get_model_list, ModelListResponse};
use serde_json::json;
use tracing::{error, info, debug};
use std::time::Instant;

#[handler]
pub async fn get_models(res: &mut Response) {
    info!("📋 收到獲取模型列表請求");
    let start_time = Instant::now();
    
    match get_model_list(Some("zh-Hant")).await {
        Ok(model_list) => {
            debug!("📊 原始模型數量: {}", model_list.data.len());
            
            let lowercase_models = ModelListResponse {
                data: model_list.data.into_iter()
                    .map(|mut model| {
                        debug!("🏷️ 處理模型: {} -> {}", model.id, model.id.to_lowercase());
                        model.id = model.id.to_lowercase();
                        model
                    })
                    .collect()
            };

            let response = json!({
                "object": "list",
                "data": lowercase_models.data
            });

            let duration = start_time.elapsed();
            info!("✅ 成功獲取模型列表 | 模型數量: {} | 處理時間: {}",
                lowercase_models.data.len(),
                crate::utils::format_duration(duration)
            );
            
            res.render(Json(response));
        },
        Err(e) => {
            let duration = start_time.elapsed();
            error!("❌ 獲取模型列表失敗 | 錯誤: {} | 耗時: {}", 
                e,
                crate::utils::format_duration(duration)
            );
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({ "error": e.to_string() })));
        }
    }
}