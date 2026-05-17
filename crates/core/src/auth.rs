/// 認証済みユーザー情報
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: ulid::Ulid,
    pub username: String,
    pub is_admin: bool,
}
