CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE users (
    id                  TEXT PRIMARY KEY,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ,
    username            TEXT NOT NULL,
    username_lower      TEXT NOT NULL,
    name                TEXT,
    bio                 TEXT,
    followers_count     INTEGER NOT NULL DEFAULT 0,
    following_count     INTEGER NOT NULL DEFAULT 0,
    notes_count         INTEGER NOT NULL DEFAULT 0,
    avatar_url          TEXT,
    banner_url          TEXT,
    is_suspended        BOOLEAN NOT NULL DEFAULT FALSE,
    is_locked           BOOLEAN NOT NULL DEFAULT FALSE,
    is_bot              BOOLEAN NOT NULL DEFAULT FALSE,
    is_cat              BOOLEAN NOT NULL DEFAULT FALSE,
    is_admin            BOOLEAN NOT NULL DEFAULT FALSE,
    location            TEXT,
    birthday            TEXT,
    lang                TEXT,
    fields              JSONB NOT NULL DEFAULT '[]',
    followed_message    TEXT,
    reaction_acceptance TEXT,
    host                TEXT,
    inbox               TEXT,
    shared_inbox        TEXT,
    featured            TEXT,
    uri                 TEXT UNIQUE,
    public_key          TEXT,
    private_key         TEXT,
    token               TEXT,
    password_hash       TEXT,
    email               TEXT,
    totp_secret         TEXT,
    totp_verified       BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE UNIQUE INDEX idx_users_username_host ON users (username_lower, COALESCE(host, ''));
CREATE INDEX idx_users_host ON users (host);
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_username_trgm ON users USING gin (username_lower gin_trgm_ops);

CREATE TABLE notes (
    id               TEXT PRIMARY KEY,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    text             TEXT,
    cw               TEXT,
    actor_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    visibility       TEXT NOT NULL DEFAULT 'public',
    renote_count     INTEGER NOT NULL DEFAULT 0,
    replies_count    INTEGER NOT NULL DEFAULT 0,
    reactions        JSONB NOT NULL DEFAULT '{}',
    reply_id         TEXT,
    renote_id        TEXT,
    file_ids         TEXT[] NOT NULL DEFAULT '{}',
    tags             TEXT[] NOT NULL DEFAULT '{}',
    has_poll         BOOLEAN NOT NULL DEFAULT FALSE,
    uri              TEXT,
    host             TEXT,
    mentions         TEXT[] NOT NULL DEFAULT '{}',
    emojis           TEXT[] NOT NULL DEFAULT '{}',
    visible_user_ids TEXT[] NOT NULL DEFAULT '{}'
);
CREATE INDEX idx_notes_actor ON notes (actor_id);
CREATE INDEX idx_notes_visibility_host_id ON notes (visibility, host, id DESC);
CREATE INDEX idx_notes_tags ON notes USING gin (tags);
CREATE INDEX idx_notes_reply ON notes (reply_id);
CREATE INDEX idx_notes_renote ON notes (renote_id);
CREATE INDEX idx_notes_uri ON notes (uri);
CREATE INDEX idx_notes_text_trgm ON notes USING gin (text gin_trgm_ops);

CREATE TABLE follows (
    follower_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    followee_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_accepted BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (follower_id, followee_id)
);
CREATE INDEX idx_follows_followee ON follows (followee_id, follower_id);

CREATE TABLE blocks (
    blocker_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker_id, blocked_id)
);
CREATE INDEX idx_blocks_blocked ON blocks (blocked_id, blocker_id);

CREATE TABLE mutes (
    muter_id   TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    muted_id   TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    PRIMARY KEY (muter_id, muted_id)
);

CREATE TABLE user_note_pins (
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    note_id    TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, note_id)
);

CREATE TABLE note_reactions (
    id         TEXT PRIMARY KEY,
    note_id    TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    actor_id   TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reaction   TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_remote  BOOLEAN NOT NULL DEFAULT FALSE,
    uri        TEXT
);
CREATE UNIQUE INDEX idx_note_reactions_unique ON note_reactions (note_id, actor_id);
CREATE INDEX idx_note_reactions_actor ON note_reactions (actor_id);

CREATE TABLE notifications (
    id                TEXT PRIMARY KEY,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    notification_type TEXT NOT NULL,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notifier_id       TEXT REFERENCES users(id) ON DELETE CASCADE,
    note_id           TEXT REFERENCES notes(id) ON DELETE CASCADE,
    reaction          TEXT,
    is_read           BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX idx_notifications_user_created ON notifications (user_id, created_at DESC);
CREATE INDEX idx_notifications_user_read ON notifications (user_id, is_read);

CREATE TABLE polls (
    id          TEXT PRIMARY KEY,
    note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ,
    multiple    BOOLEAN NOT NULL DEFAULT FALSE,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    choices     JSONB NOT NULL DEFAULT '[]'
);
CREATE INDEX idx_polls_note ON polls (note_id);

CREATE TABLE poll_votes (
    id           TEXT PRIMARY KEY,
    poll_id      TEXT NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
    actor_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    choice_index INTEGER NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX idx_poll_votes_unique ON poll_votes (poll_id, actor_id, choice_index);

CREATE TABLE drive_files (
    id            TEXT PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    owner_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    mime_type     TEXT NOT NULL,
    size          BIGINT NOT NULL,
    hash          TEXT NOT NULL,
    url           TEXT,
    thumbnail_url TEXT
);
CREATE INDEX idx_drive_files_owner_created ON drive_files (owner_id, created_at DESC);
CREATE INDEX idx_drive_files_hash ON drive_files (hash);

CREATE TABLE push_subscriptions (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint   TEXT NOT NULL UNIQUE,
    p256dh     TEXT NOT NULL,
    auth       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_push_subscriptions_user ON push_subscriptions (user_id);

CREATE TABLE relays (
    id         TEXT PRIMARY KEY,
    inbox      TEXT NOT NULL UNIQUE,
    status     TEXT NOT NULL DEFAULT 'requesting',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);
CREATE INDEX idx_relays_status ON relays (status);

CREATE TABLE activities (
    id            TEXT PRIMARY KEY,
    uri           TEXT NOT NULL UNIQUE,
    activity_type TEXT NOT NULL,
    actor_id      TEXT,
    note_id       TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE emojis (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    url        TEXT NOT NULL,
    category   TEXT,
    aliases    TEXT[] NOT NULL DEFAULT '{}',
    is_public  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE remote_emojis (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    url        TEXT NOT NULL,
    host       TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX idx_remote_emojis_name_host ON remote_emojis (name, COALESCE(host, ''));

CREATE TABLE bookmarks (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    note_id    TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX idx_bookmarks_unique ON bookmarks (user_id, note_id);
