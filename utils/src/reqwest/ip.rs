//! IP 相关的工具

use crate::reqwest;

use anyhow::{bail, Result};

/// 坐标
#[derive(Debug, Default)]
pub struct Coordinates {
    /// IP 地址
    pub addr: String,
    /// 纬度
    pub lat: Option<f64>,
    /// 经度
    pub lon: Option<f64>,
}

#[allow(dead_code)]
/// 通过IP获取地址和经纬度
pub async fn get_coordinates_from_ip(ip_address: &str) -> Result<Coordinates> {
    let mut coordinates = Coordinates::default();

    // 获取经纬度
    let url = format!(
        "http://ip-api.com/json/{}?fields=61439&lang=zh-CN",
        ip_address
    );
    let response = reqwest::get(&url).await?;

    let text = String::from_utf8_lossy(&response);
    let response = serde_json::from_str::<serde_json::Value>(&text.into_owned()).unwrap();
    let response = match response.as_object() {
        Some(v) => v,
        None => bail!("响应数据错误: {:?}", response),
    };
    let status = match response.get("status") {
        Some(v) => match v.as_str() {
            Some(v) => v,
            None => bail!("响应数据status转换错误: {:?}", response),
        },
        None => bail!("响应数据没有status: {:?}", response),
    };
    if status == "fail" {
        coordinates.addr = "局域网".to_owned();
        return Ok(coordinates);
    }

    let country = response.get("country").unwrap().as_str().unwrap();
    let region_name = response.get("regionName").unwrap().as_str().unwrap();
    let city = response.get("city").unwrap().as_str().unwrap();
    coordinates.addr = format!("{} {} {}", country, region_name, city);

    coordinates.lat = Some(response.get("lat").unwrap().as_f64().unwrap());
    coordinates.lon = Some(response.get("lon").unwrap().as_f64().unwrap());

    Ok(coordinates)
}
