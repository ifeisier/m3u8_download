#![warn(missing_docs)]

//! 主程序

use std::time::Duration;
use tokio::{
    runtime::{Builder, Runtime},
    signal,
};

mod aes_dec;
mod video;

fn main() {
    let runtime = new_multi_thread().unwrap();
    runtime.block_on(async_main());
    runtime.block_on(async move {
        signal::ctrl_c().await.unwrap();
        println!("程序结束");
    });
}

/// 异步执行入口
async fn async_main() {
    let _ = video::get(
        "/Users/igg/Downloads/123.ts",
        "https://v.gsuus.com/play/7ax928zb/index.m3u8",
    )
    .await;
}

/// 新建多线程运行时
#[allow(dead_code)]
fn new_multi_thread() -> std::io::Result<Runtime> {
    builder(
        Builder::new_multi_thread()
            .worker_threads(10)
            .thread_keep_alive(Duration::from_secs(60)),
    )
}

/// 使用当前线程新建运行时
#[allow(dead_code)]
fn new_current_thread() -> std::io::Result<Runtime> {
    builder(&mut Builder::new_current_thread())
}

/// 配置 Builder
fn builder(builder: &mut Builder) -> std::io::Result<Runtime> {
    builder
        .enable_all()
        .max_io_events_per_tick(1024)
        .max_blocking_threads(100)
        .build()
}
