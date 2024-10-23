use salvo::prelude::*;
use poe_api_process::{get_model_list, ModelListResponse};
use serde_json::json;
use tracing::{error, info};

#[handler]
pub async fn get_models(res: &mut Response) {
    info!("收到獲取模型列表請求");
    
    match get_model_list(Some("zh-Hant")).await {
        Ok(model_list) => {
            // 將所有模型 ID 轉換為小寫
            let lowercase_models = ModelListResponse {
                data: model_list.data.into_iter()
                    .map(|mut model| {
                        model.id = model.id.to_lowercase();
                        model
                    })
                    .collect()
            };

            // 建立回應物件
            let response = json!({
                "object": "list",
                "data": lowercase_models.data
            });
            info!("成功獲取模型列表");
            res.render(Json(response));
        },
        Err(e) => {
            error!("獲取模型列表失敗: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({ "error": e.to_string() })));
        }
    }
}