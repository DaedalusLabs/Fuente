use wasm_bindgen::{JsCast, JsValue};
use web_sys::{AesKeyGenParams, CryptoKey, SubtleCrypto};

pub struct BrowserCrypto {
    crypto: SubtleCrypto,
}
impl Default for BrowserCrypto {
    fn default() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let crypto = window.crypto().expect("no global `crypto` exists");
        Self {
            crypto: crypto.subtle(),
        }
    }
}
impl BrowserCrypto {
    pub async fn crypto_key_from_bytes(&self, p_key: &[u8; 32]) -> Result<CryptoKey, JsValue> {
        let array = js_sys::Uint8Array::from(&p_key[..]);
        let key_object: js_sys::Object = array.buffer().into();
        let algo = AesKeyGenParams::new("AES-GCM", 256);
        let usage_tags: js_sys::Array =
            vec![JsValue::from_str("encrypt"), JsValue::from_str("decrypt")]
                .iter()
                .collect();
        let key =
            self.crypto
                .import_key_with_object("raw", &key_object, &algo, true, &usage_tags)?;
        let key: JsValue = wasm_bindgen_futures::JsFuture::from(key).await?;
        Ok(key.dyn_into()?)
    }
    pub async fn crypto_key_to_hex(&self, js_value: CryptoKey) -> Result<String, JsValue> {
        let key =
            wasm_bindgen_futures::JsFuture::from(self.crypto.export_key("raw", &js_value)?).await?;
        let key_array: js_sys::ArrayBuffer = key.into();
        let key_array = js_sys::Uint8Array::new(&key_array);
        let key_array = key_array.to_vec();
        Ok(key_array
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
    #[wasm_bindgen_test]
    async fn _test_crypto_key_from_bytes() {
        let crypto = BrowserCrypto::default();
        let key = crypto.crypto_key_from_bytes(&[0; 32]).await.unwrap();
        assert_eq!(key.type_(), "secret");
    }
    #[wasm_bindgen_test]
    async fn _test_crypto_key_to_hex() {
        let crypto = BrowserCrypto::default();
        let key = crypto.crypto_key_from_bytes(&[0; 32]).await.unwrap();
        let hex = crypto.crypto_key_to_hex(key).await.unwrap();
        assert_eq!(hex.len(), 64);
    }
}
