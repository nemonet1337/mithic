-- Mithic SurrealDB Schema Definition
-- Version: 1.0.0
-- Description: Complete database schema for Mithic ActivityPub server

-- ============================================
-- DRIVE FOLDER TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS drive_folder SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON drive_folder TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON drive_folder TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON drive_folder TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS name ON drive_folder TYPE string;
DEFINE FIELD IF NOT EXISTS owner_id ON drive_folder TYPE string;
DEFINE FIELD IF NOT EXISTS parent_id ON drive_folder TYPE option<string>;

-- Drive folder indexes
DEFINE INDEX IF NOT EXISTS idx_drive_folder_owner ON drive_folder COLUMNS owner_id;
DEFINE INDEX IF NOT EXISTS idx_drive_folder_parent ON drive_folder COLUMNS parent_id;

-- ============================================
-- FOLLOW REQUEST TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS follow_request SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON follow_request TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON follow_request TYPE datetime;
DEFINE FIELD IF NOT EXISTS follower_id ON follow_request TYPE string;
DEFINE FIELD IF NOT EXISTS followee_id ON follow_request TYPE string;
DEFINE FIELD IF NOT EXISTS request_message ON follow_request TYPE option<string>;

-- Follow request indexes
DEFINE INDEX IF NOT EXISTS idx_follow_request_followee ON follow_request COLUMNS followee_id;
DEFINE INDEX IF NOT EXISTS idx_follow_request_follower ON follow_request COLUMNS follower_id;
DEFINE INDEX IF NOT EXISTS idx_follow_request_pair ON follow_request COLUMNS follower_id, followee_id UNIQUE;

-- ============================================
-- USER TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON user TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON user TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON user TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
DEFINE FIELD IF NOT EXISTS username_lower ON user TYPE string;
DEFINE FIELD IF NOT EXISTS name ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS bio ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS email ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS token ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS followers_count ON user TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS following_count ON user TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS notes_count ON user TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS avatar_url ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS banner_url ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_suspended ON user TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS is_locked ON user TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS is_bot ON user TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS is_admin ON user TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS host ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS inbox ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS shared_inbox ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS featured ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS uri ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS public_key ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS private_key ON user TYPE option<string>;

-- User indexes
DEFINE INDEX IF NOT EXISTS idx_user_username_lower ON user COLUMNS username_lower UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_user_host ON user COLUMNS host;
DEFINE INDEX IF NOT EXISTS idx_user_created_at ON user COLUMNS created_at;

-- ============================================
-- NOTE TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS note SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON note TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON note TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON note TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS text ON note TYPE option<string>;
DEFINE FIELD IF NOT EXISTS cw ON note TYPE option<string>;
DEFINE FIELD IF NOT EXISTS actor_id ON note TYPE record<user>;
DEFINE FIELD IF NOT EXISTS visibility ON note TYPE string DEFAULT 'public';
DEFINE FIELD IF NOT EXISTS renote_count ON note TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS replies_count ON note TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS reactions ON note TYPE object DEFAULT {};
DEFINE FIELD IF NOT EXISTS reply_id ON note TYPE option<record<note>>;
DEFINE FIELD IF NOT EXISTS renote_id ON note TYPE option<record<note>>;
DEFINE FIELD IF NOT EXISTS file_ids ON note TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS visible_user_ids ON note TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS mentions ON note TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS emojis ON note TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS tags ON note TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS has_poll ON note TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS actor_host ON note TYPE option<string>;
DEFINE FIELD IF NOT EXISTS reply_actor_id ON note TYPE option<record<user>>;
DEFINE FIELD IF NOT EXISTS renote_actor_id ON note TYPE option<record<user>>;
DEFINE FIELD IF NOT EXISTS uri ON note TYPE option<string>;

-- Note indexes
DEFINE INDEX IF NOT EXISTS idx_note_actor_id ON note COLUMNS actor_id;
DEFINE INDEX IF NOT EXISTS idx_note_created_at ON note COLUMNS created_at;
DEFINE INDEX IF NOT EXISTS idx_note_visibility_created ON note COLUMNS visibility, created_at;
DEFINE INDEX IF NOT EXISTS idx_note_renote_id ON note COLUMNS renote_id;
DEFINE INDEX IF NOT EXISTS idx_note_reply_id ON note COLUMNS reply_id;

-- ============================================
-- REACTION TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS reaction SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON reaction TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON reaction TYPE datetime;
DEFINE FIELD IF NOT EXISTS note_id ON reaction TYPE record<note>;
DEFINE FIELD IF NOT EXISTS actor_id ON reaction TYPE record<user>;
DEFINE FIELD IF NOT EXISTS reaction ON reaction TYPE string;
DEFINE FIELD IF NOT EXISTS is_remote ON reaction TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS uri ON reaction TYPE option<string>;

-- Reaction indexes
DEFINE INDEX IF NOT EXISTS idx_reaction_note_actor ON reaction COLUMNS note_id, actor_id UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_reaction_actor_id ON reaction COLUMNS actor_id;

-- ============================================
-- RENOTE TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS renote SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON renote TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON renote TYPE datetime;
DEFINE FIELD IF NOT EXISTS note_id ON renote TYPE record<note>;
DEFINE FIELD IF NOT EXISTS actor_id ON renote TYPE record<user>;
DEFINE FIELD IF NOT EXISTS renote_note_id ON renote TYPE record<note>;
DEFINE FIELD IF NOT EXISTS text ON renote TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_remote ON renote TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS uri ON renote TYPE option<string>;

-- Renote indexes
DEFINE INDEX IF NOT EXISTS idx_renote_note_actor ON renote COLUMNS note_id, actor_id UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_renote_actor_id ON renote COLUMNS actor_id;

-- ============================================
-- FOLLOW RELATIONSHIP (GRAPH)
-- ============================================
DEFINE TABLE IF NOT EXISTS follow TYPE RELATION IN user OUT user;
DEFINE FIELD IF NOT EXISTS created_at ON follow TYPE datetime;
DEFINE FIELD IF NOT EXISTS inbox ON follow TYPE option<string>;
DEFINE FIELD IF NOT EXISTS shared_inbox ON follow TYPE option<string>;

-- Follow indexes
DEFINE INDEX IF NOT EXISTS idx_follow_in ON follow COLUMNS in;
DEFINE INDEX IF NOT EXISTS idx_follow_out ON follow COLUMNS out;

-- ============================================
-- FOLLOW REQUEST TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS follow_request SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON follow_request TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON follow_request TYPE datetime;
DEFINE FIELD IF NOT EXISTS follower_id ON follow_request TYPE record<user>;
DEFINE FIELD IF NOT EXISTS followee_id ON follow_request TYPE record<user>;
DEFINE FIELD IF NOT EXISTS message ON follow_request TYPE option<string>;

-- Follow request indexes
DEFINE INDEX IF NOT EXISTS idx_follow_req_follower ON follow_request COLUMNS follower_id;
DEFINE INDEX IF NOT EXISTS idx_follow_req_followee ON follow_request COLUMNS followee_id;

-- ============================================
-- BLOCK RELATIONSHIP (GRAPH)
-- ============================================
DEFINE TABLE IF NOT EXISTS block TYPE RELATION IN user OUT user;
DEFINE FIELD IF NOT EXISTS created_at ON block TYPE datetime;

-- Block indexes
DEFINE INDEX IF NOT EXISTS idx_block_in ON block COLUMNS in;
DEFINE INDEX IF NOT EXISTS idx_block_out ON block COLUMNS out;

-- ============================================
-- MUTE RELATIONSHIP (GRAPH)
-- ============================================
DEFINE TABLE IF NOT EXISTS mute TYPE RELATION IN user OUT user;
DEFINE FIELD IF NOT EXISTS created_at ON mute TYPE datetime;
DEFINE FIELD IF NOT EXISTS expires_at ON mute TYPE option<datetime>;

-- Mute indexes
DEFINE INDEX IF NOT EXISTS idx_mute_in ON mute COLUMNS in;
DEFINE INDEX IF NOT EXISTS idx_mute_out ON mute COLUMNS out;

-- ============================================
-- DRIVE FILE TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS drive_file SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON drive_file TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON drive_file TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON drive_file TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS name ON drive_file TYPE string;
DEFINE FIELD IF NOT EXISTS mime_type ON drive_file TYPE string;
DEFINE FIELD IF NOT EXISTS file_type ON drive_file TYPE string DEFAULT 'other';
DEFINE FIELD IF NOT EXISTS size ON drive_file TYPE int;
DEFINE FIELD IF NOT EXISTS owner_id ON drive_file TYPE record<user>;
DEFINE FIELD IF NOT EXISTS path ON drive_file TYPE string;
DEFINE FIELD IF NOT EXISTS thumbnail_path ON drive_file TYPE option<string>;
DEFINE FIELD IF NOT EXISTS url ON drive_file TYPE option<string>;
DEFINE FIELD IF NOT EXISTS thumbnail_url ON drive_file TYPE option<string>;
DEFINE FIELD IF NOT EXISTS width ON drive_file TYPE option<int>;
DEFINE FIELD IF NOT EXISTS height ON drive_file TYPE option<int>;
DEFINE FIELD IF NOT EXISTS duration ON drive_file TYPE option<int>;
DEFINE FIELD IF NOT EXISTS hash ON drive_file TYPE string;
DEFINE FIELD IF NOT EXISTS is_public ON drive_file TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS folder_id ON drive_file TYPE option<string>;
DEFINE FIELD IF NOT EXISTS comment ON drive_file TYPE option<string>;

-- Drive file indexes
DEFINE INDEX IF NOT EXISTS idx_drive_file_owner ON drive_file COLUMNS owner_id;
DEFINE INDEX IF NOT EXISTS idx_drive_file_hash ON drive_file COLUMNS hash;
DEFINE INDEX IF NOT EXISTS idx_drive_file_created ON drive_file COLUMNS created_at;

-- ============================================
-- DRIVE FOLDER TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS drive_folder SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON drive_folder TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON drive_folder TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON drive_folder TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS name ON drive_folder TYPE string;
DEFINE FIELD IF NOT EXISTS owner_id ON drive_folder TYPE record<user>;
DEFINE FIELD IF NOT EXISTS parent_id ON drive_folder TYPE option<string>;

-- Drive folder indexes
DEFINE INDEX IF NOT EXISTS idx_drive_folder_owner ON drive_folder COLUMNS owner_id;

-- ============================================
-- NOTIFICATION TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS notification SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON notification TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON notification TYPE datetime;
DEFINE FIELD IF NOT EXISTS notification_type ON notification TYPE string;
DEFINE FIELD IF NOT EXISTS recipient_id ON notification TYPE record<user>;
DEFINE FIELD IF NOT EXISTS sender_id ON notification TYPE option<record<user>>;
DEFINE FIELD IF NOT EXISTS note_id ON notification TYPE option<record<note>>;
DEFINE FIELD IF NOT EXISTS reaction ON notification TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_read ON notification TYPE bool DEFAULT false;

-- Notification indexes
DEFINE INDEX IF NOT EXISTS idx_notification_recipient ON notification COLUMNS recipient_id;
DEFINE INDEX IF NOT EXISTS idx_notification_recipient_read ON notification COLUMNS recipient_id, is_read;
DEFINE INDEX IF NOT EXISTS idx_notification_created ON notification COLUMNS created_at;

-- ============================================
-- POLL TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS poll SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON poll TYPE string;
DEFINE FIELD IF NOT EXISTS note_id ON poll TYPE record<note>;
DEFINE FIELD IF NOT EXISTS created_at ON poll TYPE datetime;
DEFINE FIELD IF NOT EXISTS expires_at ON poll TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS multiple ON poll TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS is_archived ON poll TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS choices ON poll TYPE array<object>;

-- Poll indexes
DEFINE INDEX IF NOT EXISTS idx_poll_note_id ON poll COLUMNS note_id;
DEFINE INDEX IF NOT EXISTS idx_poll_expires ON poll COLUMNS expires_at;

-- ============================================
-- POLL VOTE TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS poll_vote SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON poll_vote TYPE string;
DEFINE FIELD IF NOT EXISTS poll_id ON poll_vote TYPE record<poll>;
DEFINE FIELD IF NOT EXISTS actor_id ON poll_vote TYPE record<user>;
DEFINE FIELD IF NOT EXISTS choice_index ON poll_vote TYPE int;
DEFINE FIELD IF NOT EXISTS created_at ON poll_vote TYPE datetime;

-- Poll vote indexes
DEFINE INDEX IF NOT EXISTS idx_poll_vote_poll ON poll_vote COLUMNS poll_id;
DEFINE INDEX IF NOT EXISTS idx_poll_vote_actor ON poll_vote COLUMNS actor_id;
DEFINE INDEX IF NOT EXISTS idx_poll_vote_unique ON poll_vote COLUMNS poll_id, actor_id, choice_index UNIQUE;

-- ============================================
-- BLOCK RELATIONSHIP (GRAPH)
-- ============================================
DEFINE TABLE IF NOT EXISTS block TYPE RELATION IN user OUT user;
DEFINE FIELD IF NOT EXISTS created_at ON block TYPE datetime;

-- Block indexes
DEFINE INDEX IF NOT EXISTS idx_block_in ON block COLUMNS in;
DEFINE INDEX IF NOT EXISTS idx_block_out ON block COLUMNS out;

-- ============================================
-- MUTE RELATIONSHIP (GRAPH)
-- ============================================
DEFINE TABLE IF NOT EXISTS mute TYPE RELATION IN user OUT user;
DEFINE FIELD IF NOT EXISTS created_at ON mute TYPE datetime;
DEFINE FIELD IF NOT EXISTS expires_at ON mute TYPE option<datetime>;

-- Mute indexes
DEFINE INDEX IF NOT EXISTS idx_mute_in ON mute COLUMNS in;
DEFINE INDEX IF NOT EXISTS idx_mute_out ON mute COLUMNS out;

-- ============================================
-- HASHTAG TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS hashtag SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON hashtag TYPE string;
DEFINE FIELD IF NOT EXISTS name ON hashtag TYPE string;
DEFINE FIELD IF NOT EXISTS count ON hashtag TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS mentioned_user_ids ON hashtag TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS mentioned_users_count ON hashtag TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS mentioned_local_user_ids ON hashtag TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS mentioned_local_users_count ON hashtag TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS mentioned_remote_user_ids ON hashtag TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS mentioned_remote_users_count ON hashtag TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS created_at ON hashtag TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON hashtag TYPE datetime;

-- Hashtag indexes
DEFINE INDEX IF NOT EXISTS idx_hashtag_name ON hashtag COLUMNS name UNIQUE;

-- ============================================
-- PUSH SUBSCRIPTION TABLE
-- ============================================
DEFINE TABLE IF NOT EXISTS push_subscription SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON push_subscription TYPE record<user>;
DEFINE FIELD IF NOT EXISTS endpoint ON push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS p256dh ON push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS auth ON push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON push_subscription TYPE datetime;
DEFINE FIELD IF NOT EXISTS last_used_at ON push_subscription TYPE option<datetime>;

-- Push subscription indexes
DEFINE INDEX IF NOT EXISTS idx_push_sub_user_id ON push_subscription COLUMNS user_id;
DEFINE INDEX IF NOT EXISTS idx_push_sub_endpoint ON push_subscription COLUMNS endpoint;

-- ============================================
-- RELAY TABLE (ACTIVITYPUB RELAY SERVERS)
-- ============================================
DEFINE TABLE IF NOT EXISTS relay SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON relay TYPE string;
DEFINE FIELD IF NOT EXISTS inbox ON relay TYPE string;
DEFINE FIELD IF NOT EXISTS status ON relay TYPE string DEFAULT 'requesting';
DEFINE FIELD IF NOT EXISTS created_at ON relay TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON relay TYPE option<datetime>;

-- Relay indexes
DEFINE INDEX IF NOT EXISTS idx_relay_inbox ON relay COLUMNS inbox UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_relay_status ON relay COLUMNS status;

-- ============================================
-- USER LIST TABLES
-- ============================================
DEFINE TABLE IF NOT EXISTS user_list SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON user_list TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON user_list TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON user_list TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON user_list TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON user_list TYPE option<datetime>;

-- User list indexes
DEFINE INDEX IF NOT EXISTS idx_user_list_user_id ON user_list COLUMNS user_id;

-- User list membership table
DEFINE TABLE IF NOT EXISTS user_list_membership SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON user_list_membership TYPE string;
DEFINE FIELD IF NOT EXISTS list_id ON user_list_membership TYPE record<user_list>;
DEFINE FIELD IF NOT EXISTS user_id ON user_list_membership TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON user_list_membership TYPE datetime;

-- Membership indexes
DEFINE INDEX IF NOT EXISTS idx_list_membership_list_id ON user_list_membership COLUMNS list_id;
DEFINE INDEX IF NOT EXISTS idx_list_membership_user_id ON user_list_membership COLUMNS user_id;
DEFINE INDEX IF NOT EXISTS idx_list_membership_unique ON user_list_membership COLUMNS list_id, user_id UNIQUE;

-- ============================================
-- CLIP TABLES
-- ============================================
DEFINE TABLE IF NOT EXISTS clip SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON clip TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON clip TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON clip TYPE string;
DEFINE FIELD IF NOT EXISTS is_public ON clip TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS description ON clip TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON clip TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON clip TYPE option<datetime>;

-- Clip indexes
DEFINE INDEX IF NOT EXISTS idx_clip_user_id ON clip COLUMNS user_id;
DEFINE INDEX IF NOT EXISTS idx_clip_public ON clip COLUMNS user_id, is_public;

-- Clip note table
DEFINE TABLE IF NOT EXISTS clip_note SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON clip_note TYPE string;
DEFINE FIELD IF NOT EXISTS clip_id ON clip_note TYPE record<clip>;
DEFINE FIELD IF NOT EXISTS note_id ON clip_note TYPE record<note>;
DEFINE FIELD IF NOT EXISTS created_at ON clip_note TYPE datetime;

-- Clip note indexes
DEFINE INDEX IF NOT EXISTS idx_clip_note_clip_id ON clip_note COLUMNS clip_id;
DEFINE INDEX IF NOT EXISTS idx_clip_note_note_id ON clip_note COLUMNS note_id;
DEFINE INDEX IF NOT EXISTS idx_clip_note_unique ON clip_note COLUMNS clip_id, note_id UNIQUE;

-- ============================================
-- META TABLE (INSTANCE SETTINGS)
-- ============================================
DEFINE TABLE IF NOT EXISTS meta SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON meta TYPE string;
DEFINE FIELD IF NOT EXISTS name ON meta TYPE option<string>;
DEFINE FIELD IF NOT EXISTS description ON meta TYPE option<string>;
DEFINE FIELD IF NOT EXISTS blocked_hosts ON meta TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS cache_remote_files ON meta TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS remote_drive_capacity_mb ON meta TYPE int DEFAULT 32;
DEFINE FIELD IF NOT EXISTS max_note_text_length ON meta TYPE int DEFAULT 8192;
DEFINE FIELD IF NOT EXISTS max_file_size_mb ON meta TYPE int DEFAULT 100;

-- ============================================
-- FEDERATION QUEUE (FOR BACKGROUND JOBS)
-- ============================================
DEFINE TABLE IF NOT EXISTS federation_queue SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON federation_queue TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON federation_queue TYPE datetime;
DEFINE FIELD IF NOT EXISTS inbox_url ON federation_queue TYPE string;
DEFINE FIELD IF NOT EXISTS activity ON federation_queue TYPE object;
DEFINE FIELD IF NOT EXISTS attempts ON federation_queue TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS last_attempt_at ON federation_queue TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS error_message ON federation_queue TYPE option<string>;
DEFINE FIELD IF NOT EXISTS status ON federation_queue TYPE string DEFAULT 'pending';

-- Federation queue indexes
DEFINE INDEX IF NOT EXISTS idx_fed_queue_status ON federation_queue COLUMNS status;
DEFINE INDEX IF NOT EXISTS idx_fed_queue_created ON federation_queue COLUMNS created_at;

-- ============================================
-- CHART TABLES (STATISTICS)
-- ============================================
DEFINE TABLE IF NOT EXISTS chart_entry SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON chart_entry TYPE string;
DEFINE FIELD IF NOT EXISTS chart_type ON chart_entry TYPE string;
DEFINE FIELD IF NOT EXISTS span ON chart_entry TYPE string;
DEFINE FIELD IF NOT EXISTS date ON chart_entry TYPE option<date>;
DEFINE FIELD IF NOT EXISTS hour ON chart_entry TYPE option<int>;
DEFINE FIELD IF NOT EXISTS user_id ON chart_entry TYPE option<record<user>>;
DEFINE FIELD IF NOT EXISTS total ON chart_entry TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS inc ON chart_entry TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS dec ON chart_entry TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS data ON chart_entry TYPE option<object>;
DEFINE FIELD IF NOT EXISTS created_at ON chart_entry TYPE datetime;

-- Chart entry indexes
DEFINE INDEX IF NOT EXISTS idx_chart_type_span ON chart_entry COLUMNS chart_type, span;
DEFINE INDEX IF NOT EXISTS idx_chart_date ON chart_entry COLUMNS date;
DEFINE INDEX IF NOT EXISTS idx_chart_user ON chart_entry COLUMNS user_id, chart_type;

-- ============================================
-- ANTENNA TABLES (KEYWORD MONITORING)
-- ============================================
DEFINE TABLE IF NOT EXISTS antenna SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON antenna TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON antenna TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON antenna TYPE string;
DEFINE FIELD IF NOT EXISTS source ON antenna TYPE string DEFAULT 'all';
DEFINE FIELD IF NOT EXISTS keywords ON antenna TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS exclude_keywords ON antenna TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS users ON antenna TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS user_list_id ON antenna TYPE option<string>;
DEFINE FIELD IF NOT EXISTS case_sensitive ON antenna TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS with_replies ON antenna TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS with_renotes ON antenna TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS created_at ON antenna TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON antenna TYPE option<datetime>;

-- Antenna indexes
DEFINE INDEX IF NOT EXISTS idx_antenna_user_id ON antenna COLUMNS user_id;

-- Antenna note table
DEFINE TABLE IF NOT EXISTS antenna_note SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON antenna_note TYPE string;
DEFINE FIELD IF NOT EXISTS antenna_id ON antenna_note TYPE record<antenna>;
DEFINE FIELD IF NOT EXISTS note_id ON antenna_note TYPE record<note>;
DEFINE FIELD IF NOT EXISTS created_at ON antenna_note TYPE datetime;

-- Antenna note indexes
DEFINE INDEX IF NOT EXISTS idx_antenna_note_antenna_id ON antenna_note COLUMNS antenna_id;
DEFINE INDEX IF NOT EXISTS idx_antenna_note_note_id ON antenna_note COLUMNS note_id;
DEFINE INDEX IF NOT EXISTS idx_antenna_note_unique ON antenna_note COLUMNS antenna_id, note_id UNIQUE;

-- ============================================
-- CUSTOM EMOJI TABLES
-- ============================================
DEFINE TABLE IF NOT EXISTS emoji SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS id ON emoji TYPE string;
DEFINE FIELD IF NOT EXISTS name ON emoji TYPE string;
DEFINE FIELD IF NOT EXISTS category ON emoji TYPE string DEFAULT 'general';
DEFINE FIELD IF NOT EXISTS aliases ON emoji TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS url ON emoji TYPE string;
DEFINE FIELD IF NOT EXISTS file_id ON emoji TYPE record<drive_file>;
DEFINE FIELD IF NOT EXISTS width ON emoji TYPE option<int>;
DEFINE FIELD IF NOT EXISTS height ON emoji TYPE option<int>;
DEFINE FIELD IF NOT EXISTS is_public ON emoji TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON emoji TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON emoji TYPE option<datetime>;

-- Emoji indexes
DEFINE INDEX IF NOT EXISTS idx_emoji_name ON emoji COLUMNS name;
DEFINE INDEX IF NOT EXISTS idx_emoji_category ON emoji COLUMNS category;
DEFINE INDEX IF NOT EXISTS idx_emoji_public ON emoji COLUMNS is_public;
