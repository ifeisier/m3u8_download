//! 模拟请求工具

pub mod ip;

use anyhow::Result;
use reqwest::{
    header::{ACCEPT, CONNECTION, USER_AGENT},
    Client, RequestBuilder,
};
use std::sync::LazyLock;

static CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

/// 配置 Request
fn config_request(request: RequestBuilder) -> RequestBuilder {
    request
        .header(ACCEPT, "*/*")
        .header(CONNECTION, "Keep-Alive")
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        )
}

#[allow(dead_code)]
/// 发送 GET 请求
pub async fn get(url: &str) -> Result<String> {
    let request = config_request(CLIENT.get(url));
    Ok(request.send().await?.text().await?)
}

#[allow(dead_code)]
/// 发送 POST 请求, 消息体为 json
pub async fn post_json<T: serde::Serialize + ?Sized>(url: &str, json_data: &T) -> Result<String> {
    let mut request = config_request(CLIENT.post(url));
    request = request.json(json_data);
    Ok(request.send().await?.text().await?)
}
