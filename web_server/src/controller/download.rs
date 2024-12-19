//! 下载视频

use crate::web;
use crate::JsonResponse;

use actix_web::{get, Responder};
use regex::Regex;
use serde::Deserialize;
use tokio::fs;
use tokio::process::Command;
use utils::enc_dec::aes::Aes128Cbc;
use utils::reqwest;

/// 配置 download scope 下的接口
pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(download);
}

const BASIC_PATH: &str = "/home";
// const BASIC_PATH: &str = "./download";

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
    /// 视频组路径
    path: String,
    // linux 用户组
    user_groups: String,
    // 最终路径
    final_path: String,
    /// ts 转换后的 mp4 路径
    mp4: String,
    /// 视频下载缓存
    cache: String,
    /// 合并后的 ts 视频路径
    cache_ts: String,
    /// 要下载的 ts 视频 url
    ts_url: String,
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
        user_groups: format!("{}:{}", download_info.user, download_info.user),
        mp4: format!(
            "{}/{}/chroot/downloads/{}/cache/{}/{}.mp4",
            BASIC_PATH,
            download_info.user,
            download_info.name,
            download_info.file,
            download_info.file
        ),
        path: format!(
            "{}/{}/chroot/downloads/{}",
            BASIC_PATH, download_info.user, download_info.name
        ),
        final_path: format!(
            "{}/{}/chroot/downloads/{}/{}.mp4",
            BASIC_PATH, download_info.user, download_info.name, download_info.file
        ),
        cache: format!(
            "{}/{}/chroot/downloads/{}/cache/{}",
            BASIC_PATH, download_info.user, download_info.name, download_info.file
        ),
        cache_ts: format!(
            "{}/{}/chroot/downloads/{}/cache/{}/{}.ts",
            BASIC_PATH,
            download_info.user,
            download_info.name,
            download_info.file,
            download_info.file
        ),
        ts_url: "".to_owned(),
        enc_key: None,
        iv: None,
    };

    fs::create_dir_all(&path_info.cache).await.unwrap();
    path_info.ts_url = format!("{}/ts_url.txt", &path_info.cache);

    let result = reqwest::get(&download_info.url).await;
    if let Err(e) = result {
        log::error!("下载失败: {:?}", e);
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
    fs::write(&path_info.ts_url, &ts_url).await.unwrap();

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
    let mut command = Command::new("aria2c");
    command.current_dir(&path_info.cache);
    command.arg("-j").arg("2");
    command.arg("--max-tries").arg("10");
    command.arg("--retry-wait").arg("30");
    command.arg("-i").arg("ts_url.txt");

    let mut child = command.spawn().expect("Failed to start command");
    let output = child.wait().await.unwrap();
    if output.success() {
    } else {
        fs::remove_dir_all(&path_info.cache).await.unwrap();
        return;
    }

    merge_and_clear_cache(path_info).await;
}

/// 合并和清除缓存
async fn merge_and_clear_cache(path_info: PathInfo) {
    log::info!("合并和清除缓存: {:?}.", path_info);
    let mut file_paths = Vec::with_capacity(100);
    let t = fs::read_to_string(path_info.ts_url).await.unwrap();
    let sd = t.split("\n").collect::<Vec<_>>();
    for s in sd {
        if s == "" {
            continue;
        }
        let v: Vec<&str> = s.split('/').collect();
        let file_name = v.last().unwrap();
        let file_path = format!("{}/{}", path_info.cache, file_name);
        file_paths.push(file_path);
    }
    let mut result = Vec::with_capacity(1024);
    if let Some(enc_key) = path_info.enc_key {
        let enc_key = &enc_key.into_bytes();
        let iv = &path_info.iv.unwrap();
        for file_path in file_paths {
            let mut e = fs::read(file_path).await.unwrap();
            let r = Aes128Cbc::dec(&mut e, enc_key, iv);
            result.extend(r);
        }
    } else {
        for file_path in file_paths {
            let e = fs::read(file_path).await.unwrap();
            result.extend(e);
        }
    }
    let _ = fs::write(&path_info.cache_ts, result).await;

    // 转 mp4
    let mut command = Command::new("ffmpeg");
    command.arg("-i").arg(path_info.cache_ts);
    command.arg("-c:v").arg("copy");
    command.arg("-c:a").arg("aac");
    command.arg(&path_info.mp4);
    let mut child = command.spawn().expect("Failed to start command");
    let _ = child.wait().await.unwrap();
    let _ = fs::copy(&path_info.mp4, &path_info.final_path).await;
    let _ = fs::remove_dir_all(&path_info.cache).await;

    let mut command = Command::new("chown");
    command.arg(path_info.user_groups);
    command.arg(&path_info.path);
    command.arg(&path_info.final_path);
    let mut child = command.spawn().expect("Failed to start command");
    let _ = child.wait().await.unwrap();
}
