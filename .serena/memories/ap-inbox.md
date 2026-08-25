# ActivityPub inbox (`backend/api/src/routes/activitypub.rs`)

`process_activity` が type ごとに handler を呼ぶ。未知 type は 202。

実装済み:
- Follow: 即 follow + Accept 返送
- Undo Follow/Like
- Like / Create Note|Question / Announce / poll vote
- Delete: 所有ノートなら delete_note。object URI が actor 自身ならリモートノート削除 + `is_suspended`
- Update: 埋め込み Note/Question の text/cw/tags。Person/Service のプロフィール。URI のみは skip
- Accept/Reject: object が埋め込み Follow のときだけ。followee が activity.actor と一致必須。Accept は is_accepted=true（無ければ follow 作成）。Reject は unfollow
- Block: ローカルユーザー向けなら相互 unfollow + remote→local の block 辺

所有権: Delete/Update は `note.actor_id == remote_actor.id` または actor URI 一致。ローカル user は `host != None` で上書きしない。
