use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;
use validator::Validate;

use super::actor::ActorId;

/// ノートID
pub type NoteId = Ulid;

/// 可視性設定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoteVisibility {
    /// 公開
    Public,
    /// ホームタイムラインのみ
    Home,
    /// フォロワーのみ
    Followers,
    /// 指定ユーザー
    Specified,
}

impl Default for NoteVisibility {
    fn default() -> Self {
        NoteVisibility::Public
    }
}

/// ノートモデル
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: NoteId,

    pub created_at: DateTime<Utc>,

    /// リプライ先ノートID
    pub reply_id: Option<NoteId>,

    /// リノート先ノートID
    pub renote_id: Option<NoteId>,

    /// 本文（MFM構文含む）
    #[validate(length(max = 8192))]
    pub text: Option<String>,

    /// コンテンツ警告（CW）
    #[validate(length(max = 512))]
    pub cw: Option<String>,

    /// 投稿者ID
    pub actor_id: ActorId,

    /// リノート数
    pub renote_count: i32,

    /// リプライ数
    pub replies_count: i32,

    /// リアクション（絵文字名: カウント）
    pub reactions: HashMap<String, i32>,

    /// 可視性
    pub visibility: NoteVisibility,

    /// ActivityPub URI（リモート投稿用）
    #[validate(length(max = 512))]
    pub uri: Option<String>,

    /// 添付ファイルIDs
    pub file_ids: Vec<String>,

    /// 可視ユーザーIDs（specified時）
    pub visible_user_ids: Vec<ActorId>,

    /// メンション先ユーザーIDs
    pub mentions: Vec<ActorId>,

    /// 本文内絵文字
    pub emojis: Vec<String>,

    /// ハッシュタグ
    pub tags: Vec<String>,

    /// 投票ありフラグ
    pub has_poll: bool,

    /// 投稿者のホスト（検索最適化）
    pub actor_host: Option<String>,

    /// リプライ先ユーザーID
    pub reply_actor_id: Option<ActorId>,

    /// リノート先ユーザーID
    pub renote_actor_id: Option<ActorId>,
}

impl Note {
    /// 新しいノートを作成
    pub fn new(actor_id: ActorId, text: Option<String>, visibility: NoteVisibility) -> Self {
        let now = Utc::now();

        Self {
            id: NoteId::new(),
            created_at: now,
            reply_id: None,
            renote_id: None,
            text,
            cw: None,
            actor_id,
            renote_count: 0,
            replies_count: 0,
            reactions: HashMap::new(),
            visibility,
            uri: None,
            file_ids: Vec::new(),
            visible_user_ids: Vec::new(),
            mentions: Vec::new(),
            emojis: Vec::new(),
            tags: Vec::new(),
            has_poll: false,
            actor_host: None,
            reply_actor_id: None,
            renote_actor_id: None,
        }
    }

    /// リノートかどうか
    pub fn is_renote(&self) -> bool {
        self.renote_id.is_some()
    }

    /// リプライかどうか
    pub fn is_reply(&self) -> bool {
        self.reply_id.is_some()
    }

    /// 公開範囲が制限されているか
    pub fn is_visibility_restricted(&self) -> bool {
        matches!(
            self.visibility,
            NoteVisibility::Followers | NoteVisibility::Specified
        )
    }

    /// 指定ユーザーにのみ公開かどうか
    pub fn is_specified_visibility(&self) -> bool {
        matches!(self.visibility, NoteVisibility::Specified)
    }

    /// リアクション数を取得
    pub fn total_reactions(&self) -> i32 {
        self.reactions.values().sum()
    }

    /// 本文があるかどうか（リノート本文なしの場合もある）
    pub fn has_text(&self) -> bool {
        self.text.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }

    /// ファイルが添付されているか
    pub fn has_files(&self) -> bool {
        !self.file_ids.is_empty()
    }
}
