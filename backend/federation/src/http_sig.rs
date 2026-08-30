//! ActivityPub HTTP Signatures (draft-cavage / rsa-sha256)
//!
//! Sign (outbound delivery) and verify (inbound inbox) share this module.

use base64::prelude::*;
use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::sha2::Sha256 as RsaSha256;
use rsa::signature::{SignatureEncoding, Signer, Verifier};
use rsa::RsaPublicKey;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpSigError {
    InvalidFormat,
    UnsupportedAlgorithm,
}

impl std::fmt::Display for HttpSigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "Invalid signature format"),
            Self::UnsupportedAlgorithm => write!(f, "Unsupported algorithm"),
        }
    }
}

impl std::error::Error for HttpSigError {}

/// Parsed `Signature` header
#[derive(Debug, Clone)]
pub struct HttpSignature {
    pub key_id: String,
    pub signature: String,
    pub headers: Vec<String>,
    pub algorithm: String,
}

impl HttpSignature {
    pub fn parse(header_value: &str) -> Result<Self, HttpSigError> {
        let mut key_id = None;
        let mut signature = None;
        let mut headers = vec![
            "(request-target)".to_string(),
            "host".to_string(),
            "date".to_string(),
        ];
        let mut algorithm = "rsa-sha256".to_string();

        for part in header_value.split(',') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix("keyId=\"") {
                key_id = Some(value.trim_end_matches('"').to_string());
            } else if let Some(value) = part.strip_prefix("signature=\"") {
                signature = Some(value.trim_end_matches('"').to_string());
            } else if let Some(value) = part.strip_prefix("headers=\"") {
                headers = value
                    .trim_end_matches('"')
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else if let Some(value) = part.strip_prefix("algorithm=\"") {
                algorithm = value.trim_end_matches('"').to_string();
            }
        }

        Ok(Self {
            key_id: key_id.ok_or(HttpSigError::InvalidFormat)?,
            signature: signature.ok_or(HttpSigError::InvalidFormat)?,
            headers,
            algorithm,
        })
    }
}

/// `SHA-256=<base64>` digest header value for a body
pub fn digest_header(body: &[u8]) -> String {
    format!("SHA-256={}", BASE64_STANDARD.encode(Sha256::digest(body)))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Verify `Digest: SHA-256=...` against body
pub fn verify_digest(body: &[u8], digest_header_value: &str) -> bool {
    let Some((algo, value)) = digest_header_value.split_once('=') else {
        return false;
    };
    if !algo.eq_ignore_ascii_case("sha-256") {
        return false;
    }
    let actual = BASE64_STANDARD.encode(Sha256::digest(body));
    constant_time_eq(actual.as_bytes(), value.as_bytes())
}

/// Build signing string from the header names listed in `Signature`.
/// `lookup` resolves each header name (lowercase as in the Signature header)
/// to its value. `(request-target)` must be provided by the caller via lookup
/// or the special case below.
pub fn build_signing_string(
    signed_headers: &[String],
    request_target: &str,
    host: &str,
    date: &str,
    lookup_other: impl Fn(&str) -> Option<String>,
) -> Option<String> {
    let mut lines = Vec::with_capacity(signed_headers.len());
    for name in signed_headers {
        let value = match name.as_str() {
            "(request-target)" => request_target.to_string(),
            "host" => host.to_string(),
            "date" => date.to_string(),
            other => lookup_other(other)?,
        };
        lines.push(format!("{name}: {value}"));
    }
    Some(lines.join("\n"))
}

/// Verify RSA-SHA256 / hs2019 signature over the signing string.
/// Returns `Ok(false)` for crypto failure; `Err` for unsupported algorithm.
pub fn verify_rsa(
    public_key_pem: &str,
    algorithm: &str,
    signing_string: &str,
    signature_b64: &str,
) -> Result<bool, HttpSigError> {
    if algorithm != "rsa-sha256" && algorithm != "hs2019" {
        return Err(HttpSigError::UnsupportedAlgorithm);
    }

    let Ok(signature_bytes) = BASE64_STANDARD.decode(signature_b64.as_bytes()) else {
        return Ok(false);
    };

    let pkey = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(key) => key,
        Err(_) => match RsaPublicKey::from_pkcs1_pem(public_key_pem) {
            Ok(key) => key,
            Err(_) => return Ok(false),
        },
    };

    let verifying_key = VerifyingKey::<RsaSha256>::new(pkey);
    let Ok(signature) = Signature::try_from(signature_bytes.as_slice()) else {
        return Ok(false);
    };
    Ok(verifying_key
        .verify(signing_string.as_bytes(), &signature)
        .is_ok())
}

/// High-level inbound verification (parse is caller's responsibility).
pub fn verify_request(
    public_key_pem: &str,
    signature: &HttpSignature,
    request_target: &str,
    host: &str,
    date: &str,
    lookup_other: impl Fn(&str) -> Option<String>,
) -> Result<bool, HttpSigError> {
    let Some(signing_string) =
        build_signing_string(&signature.headers, request_target, host, date, lookup_other)
    else {
        return Ok(false);
    };
    verify_rsa(
        public_key_pem,
        &signature.algorithm,
        &signing_string,
        &signature.signature,
    )
}

/// Outbound POST signing parts for ActivityPub delivery
#[derive(Debug, Clone)]
pub struct SignedPost {
    pub date: String,
    pub digest: String,
    pub signature_header: String,
}

/// Sign a POST body for delivery to `path` on `host`.
pub fn sign_post(
    private_key: &RsaPrivateKey,
    key_id: &str,
    path: &str,
    host: &str,
    body: &[u8],
) -> SignedPost {
    let date = chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();
    let digest = digest_header(body);
    let request_target = format!("post {path}");
    let signing_string =
        format!("(request-target): {request_target}\nhost: {host}\ndate: {date}\ndigest: {digest}");

    let signing_key = SigningKey::<RsaSha256>::new(private_key.clone());
    let signature = signing_key.sign(signing_string.as_bytes());
    let signature_b64 = BASE64_STANDARD.encode(signature.to_bytes());

    let signature_header = format!(
        "keyId=\"{key_id}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{signature_b64}\""
    );

    SignedPost {
        date,
        digest,
        signature_header,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1v15::SigningKey;
    use rsa::pkcs8::EncodePublicKey;
    use rsa::signature::{RandomizedSigner, SignatureEncoding};
    use rsa::{RsaPrivateKey, RsaPublicKey};

    fn gen_keypair() -> (RsaPrivateKey, String) {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let public_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        (private_key, public_pem)
    }

    fn sign_b64(pkey: &RsaPrivateKey, data: &str) -> String {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::<RsaSha256>::new(pkey.clone());
        let signature = signing_key.sign_with_rng(&mut rng, data.as_bytes());
        BASE64_STANDARD.encode(signature.to_bytes())
    }

    fn make_sig(headers: &[&str], sig_b64: &str, algorithm: &str) -> HttpSignature {
        HttpSignature {
            key_id: "https://example.com/users/alice#main-key".to_string(),
            signature: sig_b64.to_string(),
            headers: headers.iter().map(|s| s.to_string()).collect(),
            algorithm: algorithm.to_string(),
        }
    }

    #[test]
    fn build_signing_string_orders_and_formats_headers() {
        let sig = make_sig(
            &["(request-target)", "host", "date", "digest"],
            "",
            "rsa-sha256",
        );
        let result = build_signing_string(
            &sig.headers,
            "post /inbox",
            "example.com",
            "Tue, 20 Apr 2021 02:07:55 GMT",
            |name| {
                if name == "digest" {
                    Some("SHA-256=abc".into())
                } else {
                    None
                }
            },
        )
        .unwrap();

        assert_eq!(
            result,
            "(request-target): post /inbox\n\
             host: example.com\n\
             date: Tue, 20 Apr 2021 02:07:55 GMT\n\
             digest: SHA-256=abc"
        );
    }

    #[test]
    fn build_signing_string_returns_none_for_missing_header() {
        let sig = make_sig(&["(request-target)", "digest"], "", "rsa-sha256");
        let result = build_signing_string(
            &sig.headers,
            "post /inbox",
            "example.com",
            "some-date",
            |_| None,
        );
        assert!(result.is_none());
    }

    #[test]
    fn verify_accepts_valid_signature() {
        let (pkey, public_pem) = gen_keypair();
        let request_target = "post /inbox";
        let host = "example.com";
        let date = "Tue, 20 Apr 2021 02:07:55 GMT";
        let signing_string =
            format!("(request-target): {request_target}\nhost: {host}\ndate: {date}");
        let sig_b64 = sign_b64(&pkey, &signing_string);
        let sig = make_sig(
            &["(request-target)", "host", "date"],
            &sig_b64,
            "rsa-sha256",
        );

        let ok = verify_request(&public_pem, &sig, request_target, host, date, |_| None).unwrap();
        assert!(ok);
    }

    #[test]
    fn verify_rejects_tampered_signature() {
        let (pkey, public_pem) = gen_keypair();
        let host = "example.com";
        let date = "Tue, 20 Apr 2021 02:07:55 GMT";
        let signing_string = format!("(request-target): post /inbox\nhost: {host}\ndate: {date}");
        let sig_b64 = sign_b64(&pkey, &signing_string);
        let sig = make_sig(
            &["(request-target)", "host", "date"],
            &sig_b64,
            "rsa-sha256",
        );

        let ok = verify_request(&public_pem, &sig, "post /other", host, date, |_| None).unwrap();
        assert!(!ok);
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let (pkey, _) = gen_keypair();
        let (_, other_pem) = gen_keypair();
        let host = "example.com";
        let date = "Tue, 20 Apr 2021 02:07:55 GMT";
        let request_target = "post /inbox";
        let signing_string =
            format!("(request-target): {request_target}\nhost: {host}\ndate: {date}");
        let sig_b64 = sign_b64(&pkey, &signing_string);
        let sig = make_sig(
            &["(request-target)", "host", "date"],
            &sig_b64,
            "rsa-sha256",
        );

        let ok = verify_request(&other_pem, &sig, request_target, host, date, |_| None).unwrap();
        assert!(!ok);
    }

    #[test]
    fn verify_rejects_unsupported_algorithm() {
        let (_, public_pem) = gen_keypair();
        let sig = make_sig(&["(request-target)", "host", "date"], "AAAA", "ed25519");
        let result = verify_request(
            &public_pem,
            &sig,
            "post /inbox",
            "example.com",
            "some-date",
            |_| None,
        );
        assert!(matches!(result, Err(HttpSigError::UnsupportedAlgorithm)));
    }

    #[test]
    fn verify_rejects_invalid_base64() {
        let (_, public_pem) = gen_keypair();
        let sig = make_sig(
            &["(request-target)", "host", "date"],
            "not valid base64!!!",
            "rsa-sha256",
        );
        let ok = verify_request(
            &public_pem,
            &sig,
            "post /inbox",
            "example.com",
            "some-date",
            |_| None,
        )
        .unwrap();
        assert!(!ok);
    }

    #[test]
    fn digest_roundtrip() {
        let body = b"hello activitypub";
        let header = digest_header(body);
        assert!(verify_digest(body, &header));
        assert!(!verify_digest(b"tampered", &header));
    }

    #[test]
    fn sign_post_verifies() {
        let (pkey, public_pem) = gen_keypair();
        let body = br#"{"type":"Create"}"#;
        let signed = sign_post(&pkey, "https://ex/users/a#main-key", "/inbox", "ex.com", body);

        assert!(verify_digest(body, &signed.digest));

        let sig = HttpSignature::parse(&signed.signature_header).unwrap();
        let ok = verify_request(
            &public_pem,
            &sig,
            "post /inbox",
            "ex.com",
            &signed.date,
            |name| {
                if name == "digest" {
                    Some(signed.digest.clone())
                } else {
                    None
                }
            },
        )
        .unwrap();
        assert!(ok);
    }
}
