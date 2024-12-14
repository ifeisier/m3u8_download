#![warn(missing_docs)]
//! 快速创建 web 服务.
//!
//! 配置路由参考: https://actix.rs/docs/url-dispatch

mod controller;
mod response;

use crate::response::JsonResponse;

use actix_web::{web, App, HttpServer, Responder, Scope};

/// 运行 web 服务
///
/// ## 参数
/// - addrs: ip 地址和端口, 必须能转换为 SocketAddr 实例, 比如: ("127.0.0.1", 8080).
pub async fn run<A: std::net::ToSocketAddrs>(addrs: A) -> anyhow::Result<()> {
    let app = || {
        App::new()
            // .wrap(Logger::default()) // 导入: actix_web::middleware::Logger
            .service(guard_header_json("/sys").configure(controller::sys::config))
            .service(guard_header_json("/download").configure(controller::download::config))
    };
    HttpServer::new(app)
        .workers(8)
        .max_connections(1024)
        .bind(addrs)?
        .run()
        .await?;

    Ok(())
}

/// 限制请求必须是 json 格式
fn guard_header_json(path: &str) -> Scope {
    let json_config = web::JsonConfig::default().limit(4096);
    web::scope(path)
        .app_data(json_config)
        // .guard(guard::Header("content-type", "application/json"))
        .default_service(web::route().to(not_found))
}

/// url 未找到时返回 404
async fn not_found() -> impl Responder {
    JsonResponse::<()> {
        code: 404,
        message: "URL not found".to_string(),
        data: None,
    }
}
