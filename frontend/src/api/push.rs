//! Web Push subscription API

use super::client::{ApiError, request};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionBody {
    pub endpoint: String,
    pub keys: PushKeys,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionResponse {
    pub endpoint: String,
    pub keys: PushKeys,
    pub created_at: String,
}

pub async fn subscribe(
    token: &str,
    body: &PushSubscriptionBody,
) -> Result<PushSubscriptionResponse, ApiError> {
    request("POST", "push/subscription", Some(token), Some(body)).await
}

pub async fn unsubscribe(token: &str) -> Result<(), ApiError> {
    request::<(), ()>("DELETE", "push/subscription", Some(token), None)
        .await
        .map(|_| ())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstanceMeta {
    #[serde(default)]
    vapid_public_key: Option<String>,
}

pub async fn fetch_vapid_public_key() -> Result<Option<String>, ApiError> {
    let info: InstanceMeta = request("GET", "instance", None, None::<&()>).await?;
    Ok(info.vapid_public_key)
}

/// Browser PushManager.subscribe + POST to backend. WASM only.
#[cfg(target_arch = "wasm32")]
pub async fn enable_browser_push(token: &str) -> Result<(), String> {
    fn js_err(e: wasm_bindgen::JsValue) -> String {
        format!("{e:?}")
    }
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{PushSubscriptionOptionsInit, window};

    let vapid = fetch_vapid_public_key()
        .await
        .map_err(|e| e.user_message())?
        .ok_or_else(|| "このインスタンスでは Web Push が無効です (VAPID 未設定)".to_string())?;

    let window = window().ok_or_else(|| "no window".to_string())?;
    let perm = JsFuture::from(web_sys::Notification::request_permission().map_err(js_err)?)
        .await
        .map_err(js_err)?;
    let perm_str = perm.as_string().unwrap_or_default();
    if perm_str != "granted" {
        return Err("通知の許可が拒否されました".into());
    }

    let sw = window.navigator().service_worker();
    let reg_val = JsFuture::from(sw.ready().map_err(js_err)?)
        .await
        .map_err(js_err)?;
    let reg: web_sys::ServiceWorkerRegistration = reg_val
        .dyn_into()
        .map_err(|_| "service worker registration".to_string())?;

    let push_mgr = reg.push_manager().map_err(js_err)?;

    let mut key_bytes = url_safe_base64_decode(&vapid).map_err(|e| e.to_string())?;
    let opts = PushSubscriptionOptionsInit::new();
    opts.set_user_visible_only(true);
    opts.set_application_server_key_opt_u8_slice(Some(key_bytes.as_mut_slice()));

    let sub_val = JsFuture::from(push_mgr.subscribe_with_options(&opts).map_err(js_err)?)
        .await
        .map_err(js_err)?;

    let sub: web_sys::PushSubscription = sub_val
        .dyn_into()
        .map_err(|_| "push subscription".to_string())?;

    let endpoint = sub.endpoint();
    let p256dh = sub
        .get_key(web_sys::PushEncryptionKeyName::P256dh)
        .map_err(js_err)?
        .ok_or_else(|| "missing p256dh".to_string())
        .and_then(|buf| array_buffer_to_url_b64(&buf))?;
    let auth = sub
        .get_key(web_sys::PushEncryptionKeyName::Auth)
        .map_err(js_err)?
        .ok_or_else(|| "missing auth".to_string())
        .and_then(|buf| array_buffer_to_url_b64(&buf))?;

    let body = PushSubscriptionBody {
        endpoint,
        keys: PushKeys { p256dh, auth },
    };
    subscribe(token, &body)
        .await
        .map_err(|e| e.user_message())?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn url_safe_base64_decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let mut std = s.replace('-', "+").replace('_', "/");
    while std.len() % 4 != 0 {
        std.push('=');
    }
    base64_decode(&std)
}

#[cfg(target_arch = "wasm32")]
fn array_buffer_to_url_b64(buf: &js_sys::ArrayBuffer) -> Result<String, String> {
    let u8 = js_sys::Uint8Array::new(buf);
    let mut bytes = vec![0u8; u8.length() as usize];
    u8.copy_to(&mut bytes);
    Ok(url_safe_base64_encode(&bytes))
}

#[cfg(target_arch = "wasm32")]
fn url_safe_base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push(TABLE[(n & 63) as usize] as char);
        i += 3;
    }
    let rest = data.len() - i;
    if rest == 1 {
        let n = (data[i] as u32) << 16;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
    } else if rest == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
    }
    out
}

#[cfg(target_arch = "wasm32")]
fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in input.chars() {
        if c == '=' {
            break;
        }
        let v = TABLE
            .iter()
            .position(|&x| x == c as u8)
            .ok_or("invalid base64")? as u32;
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
