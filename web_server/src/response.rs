//! 定义响应数据

use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use serde::Serialize;

/// 响应 JSON 格式的数据
#[derive(Serialize)]
pub(crate) struct JsonResponse<T>
where
    T: Serialize,
{
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> Responder for JsonResponse<T>
where
    T: Serialize,
{
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}
