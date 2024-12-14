//! 随机工具

use rand::{distributions::Alphanumeric, Rng};

/// 生成指定长度的随机字符串
pub fn generate_random_string(len: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
