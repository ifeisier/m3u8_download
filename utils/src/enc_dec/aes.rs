//! aes 加密解密

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use aes::Aes128;

// 加密
// type Aes128CbcEnc = cbc::Encryptor<Aes128>;

/// 解密
type Aes128CbcDec = cbc::Decryptor<Aes128>;

pub struct Aes128Cbc;
impl Aes128Cbc {
    // https://docs.rs/cbc/0.1.2/cbc/index.html
    pub fn dec(encrypted_data: &mut [u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
        let cipher = Aes128CbcDec::new_from_slices(key, iv)
            .map_err(|e| format!("创建解密器失败: {}", e))
            .unwrap();

        let t = cipher.decrypt_padded_mut::<Pkcs7>(encrypted_data).unwrap();
        t.to_vec()
    }
}
