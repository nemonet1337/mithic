pub mod activitypub;
pub mod admin;
pub mod antennas;
pub mod auth;
pub mod blocks;
pub mod bookmarks;
pub mod charts;
pub mod clips;
pub mod drive;
pub mod drive_folders;
pub mod emojis;
pub mod exports;
pub mod filters;
pub mod follow_requests;
pub mod hashtags;
pub mod instance;
pub mod mutes;
pub mod notifications;
pub mod oauth;
pub mod polls;
pub mod push;
pub mod relays;
pub mod search;
pub mod user_lists;
pub mod statuses;
pub mod stream;
pub mod timeline;
pub mod users;

use crate::routes::{
    activitypub, admin, antennas, auth, blocks, bookmarks, charts, clips, drive, drive_folders, emojis, exports, filters, follow_requests, hashtags, instance, mutes, notifications, oauth, polls, push, relays, search, statuses, stream, timeline, user_lists, users,
};
use tower_http::trace::TraceLayer;

use crate::{
    middleware::{auth_middleware, content_negotiation_middleware, cors::cors_layer, locale_middleware, RateLimitConfig},
    middleware::http_signature::verify_http_signature,
    state::AppState,
};

/// ルーター作成
pub fn create_router(state: AppState) -> Router {
    let rate_limiter = state.rate_limiter().clone();

    // 基本的な公開エンドポイント
    let basic_public = Router::new()
        .route("/api/v1/signin", post(auth::signin))
        .route("/api/v1/signup", post(auth::signup))
        .route("/.well-known/webfinger", get(activitypub::webfinger))
        // 公開クリップ
        .route("/api/v1/users/:id/clips", get(clips::get_user_public_clips))
        // WebSocket streaming endpoint (auth optional)
        .route("/streaming", get(stream::websocket_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            crate::middleware::rate_limit_middleware,
        ));

    // ActivityPubエンドポイント - Content-Type判定付き
    let ap_routes = Router::new()
        .route("/users/:username", get(activitypub::get_actor))
        .route("/users/:username/outbox", get(activitypub::outbox))
        .route("/users/:username/followers", get(activitypub::followers))
        .route("/users/:username/following", get(activitypub::following))
        .route("/notes/:note_id", get(activitypub::get_note))
        .route("/notes/:note_id/activity", get(activitypub::get_note_activity))
        .layer(axum::middleware::from_fn(content_negotiation_middleware));

    // Inboxエンドポイント - HTTP Signature検証必須
    let inbox_routes = Router::new()
        .route("/users/:username/inbox", post(activitypub::inbox))
        .route("/inbox", post(activitypub::shared_inbox))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            verify_http_signature
        ))
        .layer(axum::middleware::from_fn(content_negotiation_middleware));

    // 認証必須のエンドポイント
    let protected_routes = Router::new()
        // ユーザー
        .route("/api/v1/accounts/verify_credentials", get(users::verify_credentials))
        .route("/api/v1/accounts/:id", get(users::get_account))
        .route("/api/v1/accounts/:id/follow", post(users::follow_account))
        .route("/api/v1/accounts/:id/unfollow", post(users::unfollow_account))
        .route("/api/v1/accounts/:id/block", post(users::block_account))
        .route("/api/v1/accounts/:id/unblock", post(users::unblock_account))
        .route("/api/v1/accounts/:id/mute", post(users::mute_account))
        .route("/api/v1/accounts/:id/unmute", post(users::unmute_account))
        .route("/api/v1/accounts/update_credentials", patch(users::update_credentials))
        .route("/api/v1/accounts/relations", get(users::get_relations))
        .route("/api/v1/accounts/:id/followers", get(users::get_followers))
        .route("/api/v1/accounts/:id/following", get(users::get_following))
        .route("/api/v1/accounts/:id/statuses", get(users::get_user_statuses))
        .route("/api/v1/i/change-password", post(users::change_password))
        .route("/api/v1/i/regenerate-token", post(users::regenerate_token))
        .route("/api/v1/i/read-all-unread-notes", post(users::read_all_unread_notes))
        .route("/api/v1/i/update-client-setting", post(users::update_client_setting))
        .route("/api/v1/i/import-following", post(users::import_following))
        .route("/api/v1/i/import-user-lists", post(users::import_user_lists))
        .route("/api/v1/i", get(users::get_i))
        .route("/api/v1/accounts/delete", post(users::delete_account))
        // タイムライン・投稿
        .route("/api/v1/timelines/home", get(timeline::home_timeline))
        .route("/api/v1/timelines/public", get(timeline::public_timeline))
        .route("/api/v1/timelines/global", get(timeline::global_timeline))
        .route("/api/v1/timelines/list/:list_id", get(timeline::list_timeline))
        .route("/api/v1/statuses", post(statuses::create_status))
        .route("/api/v1/statuses/:id", get(statuses::get_status))
        .route("/api/v1/statuses/:id", delete(statuses::delete_status))
        .route("/api/v1/statuses/:id/favourite", post(statuses::favourite_status))
        .route("/api/v1/statuses/:id/unfavourite", post(statuses::unfavourite_status))
        .route("/api/v1/statuses/:id/reblog", post(statuses::reblog_status))
        .route("/api/v1/statuses/:id/unreblog", post(statuses::unreblog_status))
        .route("/api/v1/statuses/:id/pin", post(statuses::pin_note))
        .route("/api/v1/statuses/:id/unpin", post(statuses::unpin_note))
        .route("/api/v1/statuses/:id/react", post(statuses::react_note))
        .route("/api/v1/statuses/:id/unreact", post(statuses::unreact_note))
        .route("/api/v1/statuses/:id/reactions", get(statuses::get_reactions))
        .route("/api/v1/statuses/:id/state", get(statuses::get_note_state))
        .route("/api/v1/favorites", get(statuses::get_favorites))
        // 投稿の文脈・返信・Renote
        .route("/api/v1/notes/:id/conversation", get(statuses::get_conversation))
        .route("/api/v1/notes/:id/replies", get(statuses::get_replies))
        .route("/api/v1/notes/:id/renotes", get(statuses::get_renotes))
        .route("/api/v1/notes/mentions", get(statuses::get_mentions))
        .route("/api/v1/notes/:id/children", get(statuses::get_children))
        // ドライブ
        .route("/api/v1/drive/files", get(drive::get_drive_files))
        .route("/api/v1/drive/files", post(drive::upload_file))
        .route("/api/v1/drive/files/:id", get(drive::get_drive_file))
        .route("/api/v1/drive/files/:id", patch(drive::update_drive_file))
        .route("/api/v1/drive/files/:id", delete(drive::delete_drive_file))
        .route("/api/v1/drive", get(drive::get_drive_usage))
        // ドライブフォルダ
        .route("/api/v1/drive/folders", get(drive_folders::list_folders))
        .route("/api/v1/drive/folders", post(drive_folders::create_folder))
        .route("/api/v1/drive/folders/:id", get(drive_folders::get_folder))
        .route("/api/v1/drive/folders/:id", put(drive_folders::update_folder))
        .route("/api/v1/drive/folders/:id", delete(drive_folders::delete_folder))
        // 通知
        .route("/api/v1/notifications", get(notifications::get_notifications))
        .route("/api/v1/notifications/unread_count", get(notifications::get_unread_count))
        .route("/api/v1/notifications/:id/mark_as_read", post(notifications::mark_as_read))
        .route("/api/v1/notifications/mark_all_as_read", post(notifications::mark_all_as_read))
        // ブロック
        .route("/api/v1/blocks", get(blocks::get_blocks))
        // ミュート
        .route("/api/v1/mutes", get(mutes::get_mutes))
        // タイムライン
        .route("/api/v1/timelines/home", get(timeline::home_timeline))
        .route("/api/v1/timelines/local", get(timeline::local_timeline))
        .route("/api/v1/timelines/public", get(timeline::public_timeline))
        .route("/api/v1/timelines/hybrid", get(timeline::hybrid_timeline))
        // ユーザーリスト
        .route("/api/v1/lists", get(user_lists::get_lists))
        .route("/api/v1/lists", post(user_lists::create_list))
        .route("/api/v1/lists/:id", get(user_lists::get_list))
        .route("/api/v1/lists/:id", put(user_lists::update_list))
        .route("/api/v1/lists/:id", delete(user_lists::delete_list))
        .route("/api/v1/lists/:id/accounts", post(user_lists::add_account_to_list))
        .route("/api/v1/lists/:id/accounts/:account_id", delete(user_lists::remove_account_from_list))
        // アンテナ
        .route("/api/v1/instance/stats", get(instance::get_instance_stats))
        .route("/api/v1/admin/federation/instances", get(instance::get_federated_instances))
        .route("/api/v1/federation/followers", get(instance::get_federation_followers))
        .route("/api/v1/federation/following", get(instance::get_federation_following))
        .route("/api/v1/federation/users", get(instance::get_federation_users))
        .route("/api/v1/federation/show-instance", get(instance::show_instance))
        .route("/api/v1/meta", get(instance::get_meta))
        .route("/api/v1/stats", get(instance::get_stats))
        .route("/api/v1/antennas", post(antennas::create_antenna))
        .route("/api/v1/antennas/:id", get(antennas::get_antenna))
        .route("/api/v1/antennas/:id", put(antennas::update_antenna))
        .route("/api/v1/antennas/:id", delete(antennas::delete_antenna))
        // 管理用ユーザーAPI
        .route("/api/v1/admin/users", get(admin::show_users))
        .route("/api/v1/admin/users/:user_id", delete(admin::delete_user_account))
        .route("/api/v1/admin/users/:user_id/suspend", post(admin::suspend_user))
        .route("/api/v1/admin/users/:user_id/unsuspend", post(admin::unsuspend_user))
        .route("/api/v1/admin/queue/clear", post(admin::clear_queue))
        .route("/api/v1/admin/drive/files", get(admin::get_all_drive_files))
        .route("/api/v1/admin/drive/files/delete-all", post(admin::delete_all_files_of_a_user))
        // チャート
        .route("/api/v1/charts/notes", get(charts::notes_chart))
        .route("/api/v1/charts/users", get(charts::users_chart))
        .route("/api/v1/charts/active-users", get(charts::active_users_chart))
        // 絵文字
        .route("/api/v1/custom/emojis", get(emojis::get_emojis))
        .route("/api/v1/admin/emojis/:id/copy", post(emojis::copy_emoji))
        .route("/api/v1/admin/emojis/remote", get(emojis::list_remote_emojis))
        // 検索
        .route("/api/v1/search", get(search::search))
        // ハッシュタグ
        .route("/api/v1/tags/:tag", get(hashtags::get_hashtag))
        // エクスポート
        .route("/api/v1/i/exports", get(exports::get_exports))
        .route("/api/v1/i/exports", post(exports::create_export))
        .route("/api/v1/imports/:id", get(exports::get_import))
        // ブックマーク
        .route("/api/v1/bookmarks", get(bookmarks::get_bookmarks))
        .route("/api/v1/bookmarks/:note_id", post(bookmarks::create_bookmark))
        .route("/api/v1/bookmarks/:note_id", delete(bookmarks::delete_bookmark))
        .route("/api/v1/filters", get(filters::get_filters))
        .route("/api/v1/filters", post(filters::create_filter))
        .route("/api/v1/filters/:id", get(filters::get_filter))
        .route("/api/v1/filters/:id", put(filters::update_filter))
        .route("/api/v1/filters/:id", delete(filters::delete_filter))
        // アンケート
        .route("/api/v1/polls/:id/vote", post(polls::vote))
        // プッシュ通知
        .route("/api/v1/push", get(push::get_subscription))
        .route("/api/v1/push", post(push::create_subscription))
        .route("/api/v1/push", delete(push::delete_subscription))
        // ハッシュタグ
        .route("/api/v1/tags/trending", get(hashtags::get_trending))
        .route("/api/v1/tags/search", get(hashtags::search_hashtags))
        .route("/api/v1/tags/:tag/timeline", get(hashtags::get_hashtag_timeline))
        .route("/api/v1/tags/:tag", get(hashtags::get_hashtag))
        .route("/api/v1/tags/:tag/users", get(hashtags::get_tag_users))
        .route_layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            crate::middleware::rate_limit_middleware,
        ))
        .layer(auth_middleware());

    // リレー
    let relay_routes = Router::new()
        .route("/api/v1/relays", get(relays::get_relays))
        .route("/api/v1/relays", post(relays::create_relay))
        .route("/api/v1/relays/:id", get(relays::get_relay))
        .route("/api/v1/relays/:id", delete(relays::delete_relay));

    // 全体ルーター
    Router::new()
        .merge(basic_public)
        .merge(ap_routes)
        .merge(inbox_routes)
        .merge(protected_routes)
        .merge(relay_routes)
        .layer(axum::middleware::from_fn(locale_middleware))
        .layer(cors_layer(&state.config().cors_allowed_origins))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
