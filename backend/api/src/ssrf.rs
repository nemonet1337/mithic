//! SSRF 対策: サーバー側 fetch 前の URL / 宛先 IP 検証

use std::net::{IpAddr, ToSocketAddrs};

use mithic_core::{AppError, Result};

const MAX_FETCH_BYTES: usize = 32 * 1024 * 1024; // 32 MiB

pub fn max_fetch_bytes() -> usize {
    MAX_FETCH_BYTES
}

/// http/https のみ許可し、ホスト名を解決してプライベート IP を拒否する。
pub fn validate_public_url(raw: &str) -> Result<()> {
    let url =
        url::Url::parse(raw).map_err(|e| AppError::Validation(format!("Invalid URL: {e}")))?;

    match url.scheme() {
        "http" | "https" => {}
        other => {
            return Err(AppError::Validation(format!(
                "URL scheme not allowed: {other}"
            )));
        }
    }

    let host = url
        .host_str()
        .ok_or_else(|| AppError::Validation("URL missing host".to_string()))?;

    // リテラル IP の場合は即チェック
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(AppError::Validation(
                "URL resolves to a private or reserved address".to_string(),
            ));
        }
        return Ok(());
    }

    let port = url.port_or_known_default().unwrap_or(80);
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| AppError::Validation(format!("Failed to resolve host: {e}")))?;

    let mut any = false;
    for addr in addrs {
        any = true;
        if is_blocked_ip(addr.ip()) {
            return Err(AppError::Validation(
                "URL resolves to a private or reserved address".to_string(),
            ));
        }
    }
    if !any {
        return Err(AppError::Validation("Host did not resolve".to_string()));
    }

    Ok(())
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
                || v4.octets()[0] == 0
                // CGNAT 100.64.0.0/10
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 64)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6.is_unspecified()
                || v6.is_multicast()
                // IPv4-mapped
                || v6
                    .to_ipv4_mapped()
                    .map(|v4| is_blocked_ip(IpAddr::V4(v4)))
                    .unwrap_or(false)
        }
    }
}

/// Content-Length があれば上限チェック。無ければストリーム読みで上限を適用。
pub async fn read_body_limited(response: reqwest::Response, max_bytes: usize) -> Result<Vec<u8>> {
    if let Some(len) = response.content_length()
        && len as usize > max_bytes
    {
        return Err(AppError::Validation(format!(
            "Remote content too large (max {max_bytes} bytes)"
        )));
    }

    let mut out = Vec::new();
    let mut response = response;
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                if out.len() + chunk.len() > max_bytes {
                    return Err(AppError::Validation(format!(
                        "Remote content too large (max {max_bytes} bytes)"
                    )));
                }
                out.extend_from_slice(&chunk);
            }
            Ok(None) => break,
            Err(e) => {
                return Err(AppError::Internal(format!("Failed to read response: {e}")));
            }
        }
    }
    Ok(out)
}
