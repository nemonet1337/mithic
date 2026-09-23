use crate::config::AppConfig;
use object_store::ObjectStore;
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use std::sync::Arc;

/// `STORAGE_TYPE=s3`。互換実装 (RustFS 等) も endpoint を指す同じクライアント。
pub fn is_s3_storage(storage_type: &str) -> bool {
    storage_type.trim().eq_ignore_ascii_case("s3")
}

fn nonempty(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|s| !s.is_empty())
}

/// アプリケーション設定に基づき、統一オブジェクトストレージのクライアントを作成する
pub fn create_storage_client(config: &AppConfig) -> anyhow::Result<Arc<dyn ObjectStore>> {
    if is_s3_storage(&config.storage_type) {
        let bucket = nonempty(&config.storage_s3_bucket).unwrap_or("mithic-media");
        let region = nonempty(&config.storage_s3_region).unwrap_or("us-east-1");
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_region(region);

        // カスタム endpoint = RustFS 等の S3 互換。path-style + HTTP が要る。
        // endpoint なしは AWS 既定 (virtual-hosted, HTTPS, 認証情報チェーン)。
        if let Some(endpoint) = nonempty(&config.storage_s3_endpoint) {
            let access = nonempty(&config.storage_s3_access_key);
            let secret = nonempty(&config.storage_s3_secret_key);
            let (Some(access), Some(secret)) = (access, secret) else {
                anyhow::bail!(
                    "STORAGE_S3_ACCESS_KEY and STORAGE_S3_SECRET_KEY are required when STORAGE_S3_ENDPOINT is set"
                );
            };
            builder = builder
                .with_endpoint(endpoint)
                .with_access_key_id(access)
                .with_secret_access_key(secret)
                .with_virtual_hosted_style_request(false);
            if endpoint.starts_with("http://") {
                builder = builder.with_allow_http(true);
            }
        } else {
            if let Some(access) = nonempty(&config.storage_s3_access_key) {
                builder = builder.with_access_key_id(access);
            }
            if let Some(secret) = nonempty(&config.storage_s3_secret_key) {
                builder = builder.with_secret_access_key(secret);
            }
        }

        let store = builder.build()?;
        return Ok(Arc::new(store));
    }

    if config.storage_type.trim().eq_ignore_ascii_case("local") {
        std::fs::create_dir_all(&config.local_storage_path)?;
        let store = LocalFileSystem::new_with_prefix(&config.local_storage_path)?;
        return Ok(Arc::new(store));
    }

    anyhow::bail!(
        "STORAGE_TYPE must be local or s3, got {}",
        config.storage_type
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use object_store::ObjectStoreExt;

    fn config() -> AppConfig {
        AppConfig {
            surrealdb_endpoint: "ws://localhost:8000".into(),
            surrealdb_namespace: "mithic".into(),
            surrealdb_database: "main".into(),
            surrealdb_username: "root".into(),
            surrealdb_password: "root".into(),
            surrealdb_pool_size: 1,
            dragonfly_url: "redis://localhost:6379".into(),
            jwt_secret: "test".into(),
            jwt_expiry_hours: 1,
            server_port: 3000,
            cors_allowed_origins: vec![],
            trust_proxy: false,
            storage_type: "local".into(),
            local_storage_path: "./files".into(),
            storage_s3_endpoint: None,
            storage_s3_bucket: None,
            storage_s3_access_key: None,
            storage_s3_secret_key: None,
            storage_s3_region: None,
            storage_s3_public_url: None,
            instance_url: "http://localhost:3000".into(),
            instance_name: "Mithic".into(),
            vapid_private_key: None,
            vapid_contact: "mailto:a@b.c".into(),
        }
    }

    #[test]
    fn only_s3_uses_s3_client() {
        assert!(is_s3_storage("s3"));
        assert!(is_s3_storage("S3"));
        assert!(!is_s3_storage("local"));
        assert!(!is_s3_storage("minio"));
        assert!(!is_s3_storage("rustfs"));
        let mut cfg = config();
        cfg.storage_type = "minio".into();
        let err = create_storage_client(&cfg).unwrap_err().to_string();
        assert!(err.contains("local or s3"), "{err}");
    }

    #[test]
    fn custom_endpoint_requires_keys() {
        let mut cfg = config();
        cfg.storage_type = "s3".into();
        cfg.storage_s3_endpoint = Some("http://127.0.0.1:9000".into());
        let err = create_storage_client(&cfg).unwrap_err().to_string();
        assert!(err.contains("STORAGE_S3_ACCESS_KEY"), "{err}");
    }

    #[test]
    fn custom_endpoint_client_builds_without_connecting() {
        let mut cfg = config();
        cfg.storage_type = "s3".into();
        cfg.storage_s3_endpoint = Some("http://127.0.0.1:9000".into());
        cfg.storage_s3_access_key = Some("mithic".into());
        cfg.storage_s3_secret_key = Some("mithic-dev-secret".into());
        cfg.storage_s3_bucket = Some("mithic-media".into());
        create_storage_client(&cfg).unwrap();
    }

    const RUSTFS_CONTAINER: &str = "mithic-rustfs-test";
    const RUSTFS_PORT: &str = "19000";
    const RUSTFS_ACCESS: &str = "mithic";
    const RUSTFS_SECRET: &str = "mithic-dev-secret";
    const RUSTFS_BUCKET: &str = "mithic-media";

    /// Windows の `docker` は `.bat` で、CreateProcess からは見つからない。中身は WSL。
    fn docker() -> std::process::Command {
        if cfg!(windows)
            && std::env::var_os("USERPROFILE").is_some_and(|home| {
                std::path::PathBuf::from(home)
                    .join(r"AppData\Local\bin\docker.bat")
                    .is_file()
            })
        {
            let mut cmd = std::process::Command::new("wsl.exe");
            cmd.args(["-d", "Ubuntu", "--", "docker"]);
            return cmd;
        }
        std::process::Command::new("docker")
    }

    struct StopRustfs;
    impl Drop for StopRustfs {
        fn drop(&mut self) {
            let _ = docker().args(["rm", "-f", RUSTFS_CONTAINER]).output();
        }
    }

    /// Compose の rustfs は通常コメントアウト。このテストが同じイメージを起動して put/get する。
    #[tokio::test]
    #[ignore = "starts RustFS via docker"]
    async fn rustfs_put_get() {
        let _guard = StopRustfs;
        let _ = docker().args(["rm", "-f", RUSTFS_CONTAINER]).output();
        let started = docker()
            .args([
                "run",
                "-d",
                "--name",
                RUSTFS_CONTAINER,
                "-p",
                &format!("{RUSTFS_PORT}:9000"),
                "-e",
                &format!("RUSTFS_ACCESS_KEY={RUSTFS_ACCESS}"),
                "-e",
                &format!("RUSTFS_SECRET_KEY={RUSTFS_SECRET}"),
                "-e",
                "RUSTFS_ADDRESS=:9000",
                "rustfs/rustfs:latest",
                "/data",
            ])
            .status()
            .expect("docker run");
        assert!(started.success(), "docker run rustfs failed");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap();
        let health = format!("http://127.0.0.1:{RUSTFS_PORT}/health");
        let mut up = false;
        for _ in 0..60 {
            if client
                .get(&health)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false)
            {
                up = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        assert!(up, "rustfs health check timed out");

        let bucket = docker()
            .args([
                "run",
                "--rm",
                "--network",
                &format!("container:{RUSTFS_CONTAINER}"),
                "-e",
                &format!("AWS_ACCESS_KEY_ID={RUSTFS_ACCESS}"),
                "-e",
                &format!("AWS_SECRET_ACCESS_KEY={RUSTFS_SECRET}"),
                "-e",
                "AWS_DEFAULT_REGION=us-east-1",
                "amazon/aws-cli",
                "s3",
                "mb",
                &format!("s3://{RUSTFS_BUCKET}"),
                "--endpoint-url",
                "http://127.0.0.1:9000",
            ])
            .output()
            .expect("docker run aws");
        assert!(
            bucket.status.success(),
            "mc mb failed: {}",
            String::from_utf8_lossy(&bucket.stderr)
        );

        let mut cfg = config();
        cfg.storage_type = "s3".into();
        cfg.storage_s3_endpoint = Some(format!("http://127.0.0.1:{RUSTFS_PORT}"));
        cfg.storage_s3_bucket = Some(RUSTFS_BUCKET.into());
        cfg.storage_s3_access_key = Some(RUSTFS_ACCESS.into());
        cfg.storage_s3_secret_key = Some(RUSTFS_SECRET.into());
        cfg.storage_s3_region = Some("us-east-1".into());

        let store = create_storage_client(&cfg).unwrap();
        let path = object_store::path::Path::from("hello.txt");
        store
            .put(&path, b"mithic".to_vec().into())
            .await
            .expect("put");
        let got = store
            .get(&path)
            .await
            .expect("get")
            .bytes()
            .await
            .expect("bytes");
        assert_eq!(&got[..], b"mithic");
        store.delete(&path).await.expect("delete");
    }
}
