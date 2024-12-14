//! 初始化 flexi_logger 日志.
//!
//! 通过环境变量初始化:
//! - LOG_LEVEL: 设置日志级别, 默认 info 级别.
//! - LOG_FILE_SIZE: 单个日志文件大小, 默认 2097152 = 2 MB.
//! - LOG_NAME: 日志名字, 默认 log_name.

use anyhow::Result;
use flexi_logger::{
    Age, Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, LoggerHandle, Naming,
    WriteMode,
};
use log::Record;
use std::time::Duration;
use std::{env, thread};

const IS_STDERR: bool = false; // 是否将告警和错误信息输出到终端

const IS_BUFFER: bool = true; // 是否启用日志缓冲区

/// 初始化 flexi_logger
pub fn init_flexi_logger() -> Result<LoggerHandle> {
    let log_level = env::var("LOG_LEVEL").unwrap_or("debug".to_string());
    let log_file_size = env::var("LOG_FILE_SIZE")
        .unwrap_or(2097152.to_string())
        .parse::<u64>()
        .unwrap();
    let log_name = env::var("LOG_NAME").unwrap_or("log_name".to_string());

    let file_spec = FileSpec::default()
        .directory("log_files")
        .basename(log_name)
        // .discriminant(LOG_DISCRIMINANT)
        .suppress_timestamp();

    let mut logger = Logger::try_with_env_or_str(log_level)?;
    logger = logger.log_to_file(file_spec);
    logger = logger.format(log_format);

    if IS_STDERR {
        logger = logger.duplicate_to_stderr(Duplicate::Warn);
    } else {
        logger = logger.duplicate_to_stdout(Duplicate::Trace);
    }

    if IS_BUFFER {
        logger = logger.write_mode(WriteMode::BufferAndFlushWith(3072, Duration::from_secs(10)));
    } else {
        logger = logger.write_mode(WriteMode::Direct);
    }

    logger = logger
        .rotate(
            Criterion::AgeOrSize(Age::Day, log_file_size),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(3),
        )
        .append();
    Ok(logger.start()?)
}

/// 自定义日志格式
fn log_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] T[{}] {} [{}:{}] ",
        now.format("%Y-%m-%d %H:%M:%S%.6f"),
        thread::current().name().unwrap_or("<unnamed>"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
    )?;

    // #[cfg(feature = "kv")]
    // write_key_value_pairs(w, record)?;

    write!(w, "{}", &record.args())
}
