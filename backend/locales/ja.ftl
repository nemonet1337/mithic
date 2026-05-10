# Japanese localization for Mithic Backend
# Converted from original YAML locales

# Common errors
error-not-found = リソースが見つかりません
error-validation = 無効なリクエストです
error-forbidden = アクセスが拒否されました
error-internal = 内部サーバーエラー
error-database = データベースエラー
error-cache = キャッシュエラー
error-unauthorized = 認証が必要です
error-invalid-credentials = ユーザー名またはパスワードが無効です
error-username-taken = このユーザー名は既に使用されています
error-email-taken = このメールアドレスは既に使用されています

# Actor/User related
actor-not-found = ユーザーが見つかりません
actor-suspended = このアカウントは凍結されています
actor-locked = このアカウントはフォロー承認制です
actor-fetch-failed = アクターの取得に失敗しました
actor-create-failed = アクターの作成に失敗しました
actor-invalid-username = 無効なユーザー名です
actor-username-too-long = ユーザー名が長すぎます
actor-bio-too-long = プロフィールが長すぎます
actor-name-too-long = 表示名が長すぎます

# Note/Status related
note-not-found = 投稿が見つかりません
note-deleted = 投稿を削除しました
note-create-success = 投稿を作成しました
note-invalid-visibility = 無効な公開範囲設定です
note-cannot-delete-other = 他のユーザーの投稿は削除できません
note-too-long = 投稿が長すぎます
note-empty = 投稿内容がありません
note-cw-too-long = 内容警告が長すぎます
note-contains-forbidden-word = 投稿に禁止単語が含まれています
note-reply-not-found = 返信先の投稿が見つかりません
note-renote-not-found = リノート先の投稿が見つかりません
note-quote-not-found = 引用先の投稿が見つかりません
note-vote-invalid = 無効な投票です
note-poll-expired = 投票は終了しました
note-poll-no-choice = 選択してください
note-files-too-many = ファイルが多すぎます

# Follow related
follow-success = フォローしました
follow-request-sent = フォローリクエストを送信しました
follow-already-following = 既にフォローしています
follow-not-following = フォローしていません
follow-blocked = このユーザーをフォローできません
follow-locked = フォローリクエストを送信しました
follow-self = 自分自身をフォローすることはできません
unfollow-success = フォロー解除しました

# Timeline related
timeline-empty = 表示する投稿がありません
timeline-home = ホームタイムライン
timeline-public = 公開タイムライン
timeline-local = ローカルタイムライン
timeline-social = ソーシャルタイムライン
timeline-global = グローバルタイムライン
timeline-user = ユーザータイムライン

# ActivityPub related
ap-actor-not-found = ActivityPubアクターが見つかりません
ap-invalid-activity = 無効なActivityPubアクティビティです
ap-signature-invalid = HTTP署名が無効です
ap-signature-missing-header = 必要なヘッダーがありません
ap-signature-invalid-format = 署名の形式が無効です
ap-signature-verification-failed = 署名検証に失敗しました
ap-signature-digest-mismatch = Digest検証に失敗しました
ap-signature-unsupported-algorithm = サポートされていないアルゴリズムです
ap-signature-actor-key-not-found = Actorの公開鍵が見つかりません
ap-delivery-failed = アクティビティの配送に失敗しました
ap-fetch-failed = リモートリソースの取得に失敗しました
ap-parse-failed = ActivityPubオブジェクトの解析に失敗しました
ap-create-failed = Activityの作成に失敗しました
ap-delete-failed = Activityの削除に失敗しました
ap-follow-failed = フォローに失敗しました
ap-unfollow-failed = フォロー解除に失敗しました
ap-accept-failed = 承認に失敗しました
ap-reject-failed = 拒否に失敗しました
ap-announce-failed = ブーストに失敗しました
ap-like-failed = お気に入りに失敗しました
ap-unlike-failed = お気に入り解除に失敗しました
ap-update-failed = 更新に失敗しました

# Auth related
auth-signin-success = ログインしました
auth-signup-success = アカウントを作成しました
auth-invalid-token = 無効または期限切れのトークンです
auth-token-required = アクセストークンが必要です
auth-invalid-password = パスワードが無効です
auth-password-mismatch = パスワードが一致しません
auth-password-too-short = パスワードが短すぎます
auth-password-too-long = パスワードが長すぎます
auth-invalid-email = メールアドレスが無効です
auth-email-verification-required = メールアドレスの確認が必要です
auth-rate-limited = 試行回数が多すぎます。しばらくしてからお試しください
auth-session-expired = セッションが期限切れです

# Notification related
notification-follow = { $name } さんがあなたをフォローしました
notification-unfollow = { $name } さんのフォローが外れました
notification-mention = { $name } さんがあなたに言及しました
notification-reply = { $name } さんが返信しました
notification-renote = { $name } さんがリノートしました
notification-quote = { $name } さんが引用しました
notification-reaction = { $name } さんがリアクションしました
notification-poll-ended = アンケートが終了しました
notification-receive-follow-request = { $name } さんからフォローリクエストがありました
notification-follow-request-accepted = { $name } さんがフォローリクエストを承認しました
notification-group-invited = グループに招待されました
notification-app-access = アプリがアクセスしました

# Time ago
ago-seconds = { $seconds }秒前
ago-minutes = { $minutes }分前
ago-hours = { $hours }時間前
ago-days = { $days }日前
ago-weeks = { $weeks }週間前
ago-months = { $months }ヶ月前
ago-years = { $years }年前

# Visibility
visibility-public = 公開
visibility-home = ホーム
visibility-followers = フォロワー限定
visibility-specified = ダイレクト
visibility-private = 非公開
visibility-public-description = 全員に公開
visibility-home-description = ホームタイムラインに公開
visibility-followers-description = フォロワーにのみ公開
visibility-specified-description = 指定したユーザーにのみ公開
visibility-private-description = 非公開

# Content Warning
content-warning-hide = 隠す
content-warning-show = もっと見る
content-warning-chars = { $count }文字
content-warning-files = { $count }ファイル
content-warning-poll = アンケート

# Poll
poll-choice = 選択肢 { $n }
poll-choice-n = 選択肢 { $n }
poll-no-choices = 選択肢がありません
poll-only-one-choice = 選択肢は最低2つ必要です
poll-no-more-choices = これ以上追加できません
poll-can-multiple-vote = 複数回答可
poll-cannot-multiple-vote = 単一回答
poll-expired = 終了
poll-votes = { $count }票

# Files
file-upload-failed = ファイルのアップロードに失敗しました
file-delete-failed = ファイルの削除に失敗しました
file-too-large = ファイルが大きすぎます
file-invalid-type = 無効なファイル形式です
file-name-too-long = ファイル名が長すぎます
file-alt-text-too-long = 代替テキストが長すぎます

# Drive
drive-capacity-exceeded = ドライブ容量を超えました
drive-file-not-found = ファイルが見つかりません

# Admin/Instance
admin-user-not-found = ユーザーが見つかりません
admin-cannot-modify-admin = 管理者は変更できません
admin-cannot-modify-self = 自分自身は変更できません
admin-suspend-success = ユーザーを凍結しました
admin-unsuspend-success = ユーザーの凍結を解除しました
admin-delete-success = ユーザーを削除しました
admin-password-reset-success = パスワードをリセットしました
admin-email-sent = メールを送信しました

# Federation
federation-instance-not-found = インスタンスが見つかりません
federation-instance-blocked = インスタンスをブロックしました
federation-instance-silenced = インスタンスをサイレンスしました
federation-instance-suspended = インスタンスをサスペンドしました
federation-relay-added = リレーを追加しました
federation-relay-removed = リレーを削除しました
federation-relay-not-found = リレーが見つかりません
federation-inbox-url-required = Inbox URLが必要です
federation-invalid-inbox-url = 無効なInbox URLです

# Import/Export
export-in-progress = エクスポート中
export-completed = エクスポートが完了しました
export-failed = エクスポートに失敗しました
import-in-progress = インポート中
import-completed = インポートが完了しました
import-failed = インポートに失敗しました

# Search
search-no-results = 結果が見つかりません
search-invalid-query = 無効な検索クエリです
search-rate-limited = 検索レート制限中

# Lists
list-not-found = リストが見つかりません
list-name-too-long = リスト名が長すぎます
list-too-many = リストが多すぎます
list-user-already-added = 既にリストに追加されています
list-user-not-found-in-list = リストにユーザーが見つかりません

# Groups
group-not-found = グループが見つかりません
group-name-too-long = グループ名が長すぎます
group-description-too-long = グループ説明が長すぎます
group-member-not-found = メンバーが見つかりません
group-already-member = 既にメンバーです
group-not-member = メンバーではありません
group-invitation-expired = 招待が期限切れです

# Antennas
antenna-not-found = アンテナが見つかりません
antenna-name-too-long = アンテナ名が長すぎます
antenna-keywords-too-many = キーワードが多すぎます
antenna-keywords-too-long = キーワードが長すぎます

# Clip
clip-not-found = クリップが見つかりません
clip-name-too-long = クリップ名が長すぎます
clip-description-too-long = クリップ説明が長すぎます
clip-too-many = クリップが多すぎます
note-already-clipped = 既にクリップされています

# Generic
yes = はい
no = いいえ
ok = OK
cancel = キャンセル
save = 保存
delete = 削除
edit = 編集
close = 閉じる
loading = 読み込み中...
load-more = もっと読み込む
error = エラー
success = 成功
warning = 警告
info = 情報
confirm = 確認
done = 完了
back = 戻る
next = 次へ
previous = 前へ
refresh = 更新
search = 検索
create = 作成
add = 追加
remove = 削除
update = 更新
copy = コピー
paste = 貼り付け
cut = 切り取り
select = 選択
select-all = すべて選択
undo = 元に戻す
redo = やり直す
share = 共有
report = 報告
block = ブロック
unblock = ブロック解除
mute = ミュート
unmute = ミュート解除
hide = 隠す
show = 表示
pin = ピン留め
unpin = ピン留め解除
follow = フォロー
unfollow = フォロー解除
request = リクエスト
accept = 承認
reject = 拒否
welcome = ようこそ
farewell = さようなら

# Server info
server-info = サーバー情報
server-name = サーバー名
server-description = サーバー説明
server-admin = 管理者
server-rules = ルール
tos = 利用規約
privacy-policy = プライバシーポリシー

# HTTP status
http-400 = 不正なリクエスト
http-401 = 認証が必要
http-403 = アクセス拒否
http-404 = 見つかりません
http-429 = リクエスト过多
http-500 = サーバーエラー
http-502 = 不正なゲートウェイ
http-503 = サービス利用不可
