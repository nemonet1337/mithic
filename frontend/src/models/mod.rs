pub use shared::{
    CreateNoteRequest, Note, NoteVisibility, Notification, NotificationType, ReactionSummary, User,
};

pub fn sample_user(username: &str, display_name: &str) -> User {
    let mut user = User::local(username, display_name);
    user.bio = Some(
        "UI設計と植物。決めない自由を残す。ActivityPubでつながる日々を記録しています。".into(),
    );
    user.followers_count = 1290;
    user.following_count = 248;
    user.notes_count = 4381;
    user
}

pub fn sample_notes() -> Vec<Note> {
    let hana = sample_user("hana", "Hana K.");
    let riku = sample_user("riku", "Riku M.");
    let aya = sample_user("aya", "Aya T.");
    let ken = sample_user("ken_s", "Ken S.");

    vec![
        note(
            "note-1",
            hana,
            "思考の断片を書き出す。今日のUIは少しだけ違う方向へ。 #design :sparkles:",
            "2m",
            true,
        ),
        note(
            "note-2",
            riku,
            "今読んでる本のスクショ。装丁の余白の取り方が好み。 https://example.com/books",
            "14m",
            false,
        ),
        note(
            "note-3",
            aya,
            "**MFM** のレンダリングをフロント側でも軽く確認できるようにしたい。@hana 相談したいです。",
            "1h",
            false,
        ),
        note(
            "note-4",
            ken,
            "ローカルタイムラインの速度感はSNSの体感品質に直結する。WebSocket差し込みとキャッシュの両方が大事。",
            "3h",
            true,
        ),
    ]
}

#[allow(dead_code)]
pub fn sample_notifications() -> Vec<Notification> {
    let mut notes = sample_notes();
    let note = notes.remove(0);
    let sender = sample_user("riku", "Riku M.");
    vec![
        Notification {
            id: "notif-1".into(),
            created_at: "2m".into(),
            notification_type: NotificationType::Reaction,
            sender: Some(sender.clone()),
            note: Some(note.clone()),
            reaction: Some("🔥".into()),
            is_read: false,
        },
        Notification {
            id: "notif-2".into(),
            created_at: "12m".into(),
            notification_type: NotificationType::Reply,
            sender: Some(sample_user("aya", "Aya T.")),
            note: Some(note),
            reaction: None,
            is_read: false,
        },
        Notification {
            id: "notif-3".into(),
            created_at: "1h".into(),
            notification_type: NotificationType::Follow,
            sender: Some(sender),
            note: None,
            reaction: None,
            is_read: true,
        },
    ]
}

fn note(id: &str, author: User, content: &str, created_at: &str, accent: bool) -> Note {
    Note {
        id: id.into(),
        created_at: created_at.into(),
        author,
        content: content.into(),
        cw: None,
        visibility: NoteVisibility::Public,
        reactions: vec![
            ReactionSummary {
                emoji: "🔥".into(),
                count: 24,
                reacted_by_me: accent,
            },
            ReactionSummary {
                emoji: "✨".into(),
                count: 18,
                reacted_by_me: false,
            },
        ],
        reply_count: 12,
        renote_count: 47,
        quote_count: 4,
        attachments: Vec::new(),
        tags: vec!["design".into()],
        is_nsfw: false,
    }
}
