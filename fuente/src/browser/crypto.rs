use js_sys::{ArrayBuffer, Object, Uint8Array};
use nostro2::userkeys::UserKeys;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AesKeyGenParams, CryptoKey, SubtleCrypto};
use super::web_utils::{vec_to_js_array, window};

fn crypto_subtle() -> SubtleCrypto {
    window().crypto().unwrap().subtle()
}
fn user_keys_to_object(user_keys: &UserKeys) -> Object {
    let secret_bytes = user_keys.get_secret_key();
    let array = Uint8Array::from(&secret_bytes[..]);
    array.buffer().into()
}

pub async fn user_keys_to_crypto(user_keys: &UserKeys) -> JsValue {
    let key_object = user_keys_to_object(user_keys);
    let crypto = crypto_subtle();
    let algo = AesKeyGenParams::new("AES-GCM", 256);
    let usages = vec_to_js_array(&["encrypt", "decrypt"]);
    let key = crypto
        .import_key_with_object("raw", &key_object, &algo, true, &usages)
        .unwrap();
    let key: JsValue = JsFuture::from(key).await.unwrap();
    key
}

pub async fn crypto_to_user_keys(
    js_value: CryptoKey,
    extractable: bool,
) -> Result<UserKeys, JsValue> {
    let crypto = crypto_subtle();
    let key = JsFuture::from(crypto.export_key("raw", &js_value)?).await?;
    let key_array: ArrayBuffer = key.into();
    let key_array = Uint8Array::new(&key_array);
    let key_array = key_array.to_vec();
    let key_hex = key_array
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();
    match extractable {
        true => Ok(UserKeys::new_extractable(&key_hex).unwrap()),
        false => Ok(UserKeys::new(&key_hex).unwrap()),
    }
}
