use crate::auth::{hash_password, verify_password};
use crate::db::SurrealClient;
use crate::db::queries::{create_actor, get_actor_by_username, get_actor_by_username_or_email};
use crate::models::actor::Actor;
use crate::{AppError, Result};
use shared::SignupRequest;

/// ActivityPub 連合用の RSA-2048 鍵ペアを生成する (PEM)
fn generate_keypair() -> Result<(String, String)> {
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use rsa::{RsaPrivateKey, RsaPublicKey};

    let mut rng = rand::rngs::OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048)
        .map_err(|e| AppError::Internal(format!("Failed to generate RSA key: {e}")))?;
    let public_key = RsaPublicKey::from(&private_key);

    let private_pem = private_key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| AppError::Internal(format!("Failed to encode private key: {e}")))?
        .to_string();
    let public_pem = public_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| AppError::Internal(format!("Failed to encode public key: {e}")))?;

    Ok((private_pem, public_pem))
}

pub async fn register_user(
    surreal: &SurrealClient,
    request: SignupRequest,
    instance_url: &str,
) -> Result<Actor> {
    if get_actor_by_username(surreal, &request.username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .is_some()
    {
        return Err(AppError::Validation(
            "Username is already taken".to_string(),
        ));
    }

    let password_hash = hash_password(&request.password)?;

    // 鍵生成は CPU バウンドなのでブロッキングプールで実行
    let (private_pem, public_pem) = tokio::task::spawn_blocking(generate_keypair)
        .await
        .map_err(|e| AppError::Internal(format!("Key generation task failed: {e}")))??;

    let mut actor = Actor::new_local(request.username, request.name);
    actor.password_hash = Some(password_hash);
    actor.email = request.email.map(|e| e.to_lowercase());
    actor.private_key = Some(private_pem);
    actor.public_key = Some(public_pem);

    let actor_uri = actor.actor_uri(instance_url);
    actor.uri = Some(actor_uri.clone());
    actor.inbox = Some(actor.inbox_url(instance_url));
    actor.shared_inbox = Some(format!("{instance_url}/inbox"));

    let created = create_actor(surreal, &actor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(created)
}

pub async fn authenticate_user(
    surreal: &SurrealClient,
    username: &str,
    password: &str,
) -> Result<Actor> {
    let actor = get_actor_by_username_or_email(surreal, username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let hash = actor
        .password_hash
        .as_ref()
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    if !verify_password(password, hash)? {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    if actor.is_suspended {
        return Err(AppError::Forbidden("Account is suspended".to_string()));
    }

    Ok(actor)
}
