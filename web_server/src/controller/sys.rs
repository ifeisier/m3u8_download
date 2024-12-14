//! 获取系统状态的接口

use crate::response::JsonResponse;
use actix_web::{post, web, Responder};
use serde::Deserialize;
use std::collections::HashMap;

/// 配置 sys scope 下的接口
pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_show);
    cfg.service(post_show);
}

/// 用来做测试的
#[derive(Deserialize)]
struct Info {
    user_id: u32,
}

#[post("/show/{user_id}")]
async fn get_show(
    path: actix_web::Result<web::Path<Info>>,
    query: actix_web::Result<web::Query<Info>>,
    json: actix_web::Result<web::Json<Info>>,
    form: actix_web::Result<web::Form<Info>>,
) -> impl Responder {
    match path {
        Ok(v) => println!("Ok {}", v.user_id),
        Err(e) => println!("Err {:?}", e),
    }

    match query {
        Ok(v) => println!("Ok {}", v.user_id),
        Err(e) => println!("Err {:?}", e),
    }

    match json {
        Ok(v) => println!("Ok {}", v.user_id),
        Err(e) => println!("Err {:?}", e),
    }

    match form {
        Ok(v) => println!("Ok {}", v.user_id),
        Err(e) => println!("Err {:?}", e),
    }

    JsonResponse::<()> {
        code: 200,
        message: "get_show".to_string(),
        data: None,
    }
}

#[post("/show")]
async fn post_show() -> impl Responder {
    let current_thread = std::thread::current();
    let thread_name = current_thread.name().unwrap_or("Unnamed");

    println!("Current thread name: {}", thread_name);

    let mut sss = HashMap::new();
    sss.insert("1", "1");
    sss.insert("2", "2");

    JsonResponse::<HashMap<&str, &str>> {
        code: 200,
        message: "post_show".to_string(),
        data: Some(sss),
    }
}
