//! 用来获取和保存视频.

use reqwest::Client;
use std::{io::Write, sync::LazyLock};
use tokio::time::Duration;

use anyhow::{bail, Result};
use regex::Regex;

use crate::aes_dec;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(60 * 30))
        .build()
        .unwrap()
});

pub async fn get(name: &str, url: &str) -> Result<()> {
    let text = match CLIENT.get(url).send().await {
        Ok(v) => v.text().await?,
        Err(e) => bail!("请求:{} 错误:{:?}", url, e),
    };

    let key_regex = Regex::new(
        r#"#EXT-X-KEY:METHOD=(?P<method>[^,]+),URI="(?P<uri>[^"]+)",IV=(?P<iv>0x[0-9A-Fa-f]+)"#,
    )?;
    let key_caps = match key_regex.captures(&text) {
        Some(v) => v,
        None => bail!("未找到 EXT-X-KEY 信息"),
    };

    let enc_key_url = {
        key_caps.name("uri").and_then(|v| {
            let uri = v.as_str();
            if uri.starts_with("https://") {
                Some(uri.to_owned())
            } else {
                Some(url.replace("index.m3u8", uri))
            }
        })
    };
    let mut enc_key = Vec::<u8>::new();
    if let Some(enc_key_url) = enc_key_url {
        let t = CLIENT.get(&enc_key_url).send().await?;
        enc_key = t.bytes().await?.to_vec();
    }

    let method = key_caps.name("method").unwrap().as_str();
    let iv = key_caps.name("iv").unwrap().as_str().replace("0x", "");
    let iv = hex::decode(iv).unwrap();

    println!("method: {}", method);
    println!("enc_key: {:?}", enc_key);
    println!("iv: {:?}", iv);

    let url_regex = Regex::new(r#"https://[^\s]+"#).unwrap();
    let urls: Vec<String> = url_regex
        .find_iter(&text)
        .map(|mat| mat.as_str().to_owned())
        .collect();
    println!("urls: {:?}", urls);

    let download_tasks: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let enc_key = enc_key.clone();
            let iv = iv.clone();
            tokio::spawn(async move {
                match CLIENT.get(&url).send().await {
                    Ok(response) => {
                        let mut data = response.bytes().await.unwrap().to_vec();
                        Ok(aes_dec::dec(&mut data, &enc_key, &iv))
                    }
                    // Ok(response) => Ok(format!("{}: {}", url, response.status())),
                    Err(e) => bail!("下载 {} 失败: {:?}", url, e),
                }
            })
        })
        .collect();

    let mut file = std::fs::File::create(name).unwrap();
    for task in download_tasks {
        match task.await {
            Ok(Ok(content)) => {
                let len = file.write(&content);
                println!("下载成功: {:?}", len)
            }
            Ok(Err(e)) => println!("下载失败: {:?}", e),
            Err(e) => println!("任务失败: {:?}", e),
        }
    }
    let clone = file.try_clone();
    println!("file: {:?}", clone);

    println!("下载完成");
    Ok(())
}
