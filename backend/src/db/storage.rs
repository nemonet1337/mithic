use crate::config::AppConfig;
use object_store::ObjectStore;
use object_store::aws::AmazonS3Builder;
use object_store::gcp::GoogleCloudStorageBuilder;
use object_store::local::LocalFileSystem;
use std::sync::Arc;

/// アプリケーション設定に基づき、統一オブジェクトストレージのクライアントを作成する
pub fn create_storage_client(config: &AppConfig) -> anyhow::Result<Arc<dyn ObjectStore>> {
    match config.storage_type.as_str() {
        "s3" | "minio" | "r2" => {
            let bucket = config
                .storage_s3_bucket
                .as_deref()
                .unwrap_or("mithic-media");

            let mut builder = AmazonS3Builder::new().with_bucket_name(bucket);

            if let Some(ref endpoint) = config.storage_s3_endpoint {
                builder = builder.with_endpoint(endpoint);
                builder = builder.with_allow_http(true);
            }
            if let Some(ref access_key) = config.storage_s3_access_key {
                builder = builder.with_access_key_id(access_key);
            }
            if let Some(ref secret_key) = config.storage_s3_secret_key {
                builder = builder.with_secret_access_key(secret_key);
            }
            if let Some(ref region) = config.storage_s3_region {
                builder = builder.with_region(region);
            }

            // MinIOなどのローカル環境では virtual_hosted_style ではなく path_style 接続が必要
            builder = builder.with_virtual_hosted_style_request(false);

            let store = builder.build()?;
            Ok(Arc::new(store))
        }
        "gcs" => {
            let bucket = config
                .storage_gcs_bucket
                .as_deref()
                .unwrap_or("mithic-media");

            let mut builder = GoogleCloudStorageBuilder::new().with_bucket_name(bucket);

            if let Some(ref credentials) = config.storage_gcs_credentials {
                if credentials.starts_with('{') {
                    builder = builder.with_service_account_key(credentials);
                } else {
                    builder = builder.with_service_account_path(credentials);
                }
            }

            let store = builder.build()?;
            Ok(Arc::new(store))
        }
        _ => {
            // デフォルトはローカルファイルシステム
            std::fs::create_dir_all(&config.local_storage_path)?;
            let store = LocalFileSystem::new_with_prefix(&config.local_storage_path)?;
            Ok(Arc::new(store))
        }
    }
}
