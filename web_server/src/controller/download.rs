//! 下载视频

use crate::web;
use crate::JsonResponse;

use actix_web::{get, Responder};
use regex::Regex;
use serde::Deserialize;
use tokio::fs;
use tokio::process::Command;
use utils::enc_dec::aes::Aes128CbcDec;
use utils::reqwest;

/// 配置 download scope 下的接口
pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(download);
}

const BASIC_PATH: &str = "D:\\0.项目\\m3u8_download";

/// 下载信息
#[derive(Deserialize, Debug)]
struct DownloadInfo {
    url: String,
    name: String,
    file: String,
    user: String,
}

/// 路径信息
#[derive(Deserialize, Debug)]
struct PathInfo {
    basic: String,
    cache: String,
    ts_url_path: String,
    enc_key: Option<String>,
    iv: Option<Vec<u8>>,
}

#[get("")]
async fn download(query: actix_web::Result<web::Query<DownloadInfo>>) -> impl Responder {
    if query.is_err() {
        return JsonResponse::<()> {
            code: 400,
            message: "参数错误".to_string(),
            data: None,
        };
    }
    let download_info = query.unwrap();

    let mut path_info = PathInfo {
        basic: format!(
            "{}/{}/{}/{}",
            BASIC_PATH, download_info.user, download_info.name, download_info.file
        ),
        cache: format!(
            "{}/{}/{}/cache/{}",
            BASIC_PATH, download_info.user, download_info.name, download_info.file
        ),
        ts_url_path: "".to_owned(),
        enc_key: None,
        iv: None,
    };

    fs::create_dir_all(&path_info.cache).await.unwrap();
    path_info.ts_url_path = format!("{}/ts_url.txt", &path_info.cache);

    let result = reqwest::get(&download_info.url).await;
    if let Err(e) = result {
        log::error!("下载失败: {}", e);
        return JsonResponse::<()> {
            code: 400,
            message: "下载失败".to_string(),
            data: None,
        };
    }
    let m3u8 = result.unwrap();

    // 提取 ts url 连接
    // let mut ts_url = Vec::new();
    let mut ts_url = "".to_owned();
    let ts_regex = Regex::new(r"https?://[^\s]+\.ts").unwrap();
    for cap in ts_regex.captures_iter(&m3u8) {
        ts_url.push_str(&cap[0].to_owned());
        ts_url.push('\n');
    }
    fs::write(&path_info.ts_url_path, &ts_url).await.unwrap();

    let key_regex = Regex::new(
        r#"#EXT-X-KEY:METHOD=(?P<method>[^,]+),URI="(?P<uri>[^"]+)",IV=(?P<iv>0x[0-9A-Fa-f]+)"#,
    )
    .unwrap();

    let key_caps = key_regex.captures(&m3u8);
    let enc_key_url = key_caps.as_ref().and_then(|key_caps| {
        key_caps.name("uri").and_then(|v| {
            let uri = v.as_str();
            if uri.starts_with("https://") {
                Some(uri.to_owned())
            } else {
                Some(download_info.url.replace("index.m3u8", uri))
            }
        })
    });
    if let Some(enc_key_url) = enc_key_url {
        match reqwest::get(&enc_key_url).await {
            Ok(v) => {
                path_info.enc_key = Some(v);
                path_info.iv = Some(
                    hex::decode(
                        key_caps
                            .unwrap()
                            .name("iv")
                            .unwrap()
                            .as_str()
                            .replace("0x", ""),
                    )
                    .unwrap(),
                );
            }
            Err(e) => {
                log::error!("下载失败: {}", e);
                return JsonResponse::<()> {
                    code: 400,
                    message: "下载失败".to_string(),
                    data: None,
                };
            }
        }
    }

    tokio::spawn(async move {
        log::info!("准备下载: {:?}.", path_info);
        create_download_task(path_info).await;
    });
    JsonResponse::<()> {
        code: 200,
        message: "创建下载任务成功".to_string(),
        data: None,
    }
}

/// 创建下载任务
async fn create_download_task(path_info: PathInfo) {
    // let mut command = Command::new("aria2c.exe");
    // command.current_dir(&path);
    // command.arg("-j").arg("2");
    // command.arg("--max-tries").arg("10");
    // command.arg("--retry-wait").arg("30");
    // command.arg("-i").arg("ts_url.txt");

    // let mut child = command.spawn().expect("Failed to start command");
    // let output = child.wait().await.unwrap();
    // if output.success() {

    // } else {
    //     fs::remove_dir_all(&path).await.unwrap();
    //     return;
    // }

    // 获取文件绝对路径
    let mut file_paths = Vec::with_capacity(100);
    let t = fs::read_to_string(path_info.ts_url_path).await.unwrap();
    let sd = t.split("\n").collect::<Vec<_>>();
    for s in sd {
        let v: Vec<&str> = s.split('/').collect();
        let file_name = v.last().unwrap();
        let file_path = format!("{}/{}", path_info.basic, file_name);
        file_paths.push(file_path);
    }
    println!("file_paths: {:?}", file_paths);

    let mut result = Vec::with_capacity(1024);
    if let Some(enc_key) = path_info.enc_key {
        let enc_key = &enc_key.into_bytes();
        let iv = &path_info.iv.unwrap();
        for file_path in file_paths {
            let mut e = fs::read(file_path).await.unwrap();
            let r = Aes128CbcDec::dec(&mut e, enc_key, iv);
            result.push(r);
        }
    } else {
        for file_path in file_paths {
            let e = fs::read(file_path).await.unwrap();
            result.push(e);
        }
    }

    // fs::write(path, result);

    // 获取 path 下的所有 ts 格式的文件
    // let ts_files = std::fs::read_dir(&path).unwrap();
    // let mut ts_file_names = Vec::new();
    // for ts_file in ts_files {
    //     let ts_file = ts_file.unwrap();
    //     let file_name = ts_file.file_name();
    //     let file_name = file_name.to_str().unwrap();
    //     if file_name.ends_with(".ts") {
    //         ts_file_names.push(file_name.to_string());
    //     }
    // }

    // ts_file_names.sort_by(|a, b| {
    //     let a_num: usize = a
    //         .trim_start_matches("plist")
    //         .trim_end_matches(".ts")
    //         .parse()
    //         .unwrap_or(0);
    //     let b_num: usize = b
    //         .trim_start_matches("plist")
    //         .trim_end_matches(".ts")
    //         .parse()
    //         .unwrap_or(0);

    //     a_num.cmp(&b_num)
    // });

    // println!("ts_file_names: {:#?}", ts_file_names);
}
