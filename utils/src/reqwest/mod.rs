//! 模拟请求工具

pub mod ip;

use anyhow::Result;
use bytes::Bytes;
use reqwest::{
    header::{ACCEPT, CONNECTION},
    Client, RequestBuilder,
};
use std::sync::LazyLock;
use std::time::Duration;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(60 * 30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()
        .unwrap()
});

/// 配置 Request
fn config_request(request: RequestBuilder) -> RequestBuilder {
    request
        .header(ACCEPT, "*/*")
        .header(CONNECTION, "Keep-Alive")
}

#[allow(dead_code)]
/// 发送 GET 请求
pub async fn get(url: &str) -> Result<Bytes> {
    let request = config_request(CLIENT.get(url));
    Ok(request.send().await?.bytes().await?)
}

#[allow(dead_code)]
/// 发送 POST 请求, 消息体为 json
pub async fn post_json<T: serde::Serialize + ?Sized>(url: &str, json_data: &T) -> Result<Bytes> {
    let mut request = config_request(CLIENT.post(url));
    request = request.json(json_data);
    Ok(request.send().await?.bytes().await?)
}
