#![warn(missing_docs)]

//! 应用程序入口

use utils::flexi_logger;
use tokio::{
    runtime::{Builder, Runtime},
    signal,
    time::Duration,
};

fn main() {
    let logger = flexi_logger::init_flexi_logger().unwrap();

    let runtime = new_multi_thread().unwrap();
    runtime.block_on(async_main());
    runtime.block_on(async move {
        signal::ctrl_c().await.unwrap();
        logger.flush();
        logger.shutdown();
    });
}

/// 异步执行入口
async fn async_main() {
    println!("Hello, world!");

    if let Err(e) = web_server::run(("0.0.0.0", 9090)).await {
        panic!("启动WebServer失败, {:?}", e)
    }
}

/// 新建多线程运行时
#[allow(dead_code)]
fn new_multi_thread() -> std::io::Result<Runtime> {
    builder(
        Builder::new_multi_thread()
            .worker_threads(50)
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
