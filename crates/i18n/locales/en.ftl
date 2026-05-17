# English localization for Mithic Backend
# Converted from original YAML locales

# Common errors
error-not-found = Resource not found
error-validation = Invalid request
error-forbidden = Access denied
error-internal = Internal server error
error-database = Database error
error-cache = Cache error
error-unauthorized = Authentication required
error-invalid-credentials = Invalid username or password
error-username-taken = Username already taken
error-email-taken = Email address already taken

# Actor/User related
actor-not-found = User not found
actor-suspended = This account has been suspended
actor-locked = This account requires follow approval
actor-fetch-failed = Failed to fetch actor
actor-create-failed = Failed to create actor
actor-invalid-username = Invalid username
actor-username-too-long = Username is too long
actor-bio-too-long = Bio is too long
actor-name-too-long = Display name is too long

# Note/Status related
note-not-found = Post not found
note-deleted = Post deleted successfully
note-create-success = Post created successfully
note-invalid-visibility = Invalid visibility setting
note-cannot-delete-other = Cannot delete another user's post
note-too-long = Post is too long
note-empty = Post cannot be empty
note-cw-too-long = Content warning is too long
note-contains-forbidden-word = Post contains forbidden words
note-reply-not-found = Reply target not found
note-renote-not-found = Renote target not found
note-quote-not-found = Quote target not found
note-vote-invalid = Invalid vote
note-poll-expired = Poll has expired
note-poll-no-choice = Please select a choice
note-files-too-many = Too many files

# Follow related
follow-success = Now following
follow-request-sent = Follow request sent
follow-already-following = Already following
follow-not-following = Not following
follow-blocked = Cannot follow this user
follow-locked = Follow request sent
follow-self = Cannot follow yourself
unfollow-success = Unfollowed

# Timeline related
timeline-empty = No posts to show
timeline-home = Home timeline
timeline-public = Public timeline
timeline-local = Local timeline
timeline-social = Social timeline
timeline-global = Global timeline
timeline-user = User timeline

# ActivityPub related
ap-actor-not-found = ActivityPub actor not found
ap-invalid-activity = Invalid ActivityPub activity
ap-signature-invalid = Invalid HTTP signature
ap-signature-missing-header = Missing required header
ap-signature-invalid-format = Invalid signature format
ap-signature-verification-failed = Signature verification failed
ap-signature-digest-mismatch = Digest verification failed
ap-signature-unsupported-algorithm = Unsupported algorithm
ap-signature-actor-key-not-found = Actor key not found
ap-delivery-failed = Activity delivery failed
ap-fetch-failed = Failed to fetch remote resource
ap-parse-failed = Failed to parse ActivityPub object
ap-create-failed = Failed to create Activity
ap-delete-failed = Failed to delete Activity
ap-follow-failed = Failed to follow
ap-unfollow-failed = Failed to unfollow
ap-accept-failed = Failed to accept
ap-reject-failed = Failed to reject
ap-announce-failed = Failed to announce
ap-like-failed = Failed to like
ap-unlike-failed = Failed to unlike
ap-update-failed = Failed to update

# Auth related
auth-signin-success = Signed in successfully
auth-signup-success = Account created successfully
auth-invalid-token = Invalid or expired token
auth-token-required = Access token required
auth-invalid-password = Invalid password
auth-password-mismatch = Passwords do not match
auth-password-too-short = Password is too short
auth-password-too-long = Password is too long
auth-invalid-email = Invalid email address
auth-email-verification-required = Email verification required
auth-rate-limited = Too many attempts. Please try again later
auth-session-expired = Session has expired

# Notification related
notification-follow = { $name } followed you
notification-unfollow = { $name } unfollowed you
notification-mention = { $name } mentioned you
notification-reply = { $name } replied to your post
notification-renote = { $name } renoted your post
notification-quote = { $name } quoted your post
notification-reaction = { $name } reacted to your post
notification-poll-ended = Poll has ended
notification-receive-follow-request = { $name } sent a follow request
notification-follow-request-accepted = { $name } accepted your follow request
notification-group-invited = You were invited to a group
notification-app-access = An app was accessed

# Time ago
ago-seconds = { $seconds ->
    [one] { $seconds } second ago
    *[other] { $seconds } seconds ago
}
ago-minutes = { $minutes ->
    [one] { $minutes } minute ago
    *[other] { $minutes } minutes ago
}
ago-hours = { $hours ->
    [one] { $hours } hour ago
    *[other] { $hours } hours ago
}
ago-days = { $days ->
    [one] { $days } day ago
    *[other] { $days } days ago
}
ago-weeks = { $weeks ->
    [one] { $weeks } week ago
    *[other] { $weeks } weeks ago
}
ago-months = { $months ->
    [one] { $months } month ago
    *[other] { $months } months ago
}
ago-years = { $years ->
    [one] { $years } year ago
    *[other] { $years } years ago
}

# Visibility
visibility-public = Public
visibility-home = Home
visibility-followers = Followers only
visibility-specified = Direct message
visibility-private = Private
visibility-public-description = Visible to all
visibility-home-description = Visible to followers and on home timeline
visibility-followers-description = Visible to followers only
visibility-specified-description = Visible to mentioned users only
visibility-private-description = Private

# Content Warning
content-warning-hide = Hide
content-warning-show = Show more
content-warning-chars = { $count } characters
content-warning-files = { $count } files
content-warning-poll = Poll

# Poll
poll-choice = Choice { $n }
poll-choice-n = Choice { $n }
poll-no-choices = No choices available
poll-only-one-choice = At least 2 choices are required
poll-no-more-choices = No more choices can be added
poll-can-multiple-vote = Multiple choices allowed
poll-cannot-multiple-vote = Only one choice allowed
poll-expired = Expired
poll-votes = { $count ->
    [one] { $count } vote
    *[other] { $count } votes
}

# Files
file-upload-failed = Failed to upload file
file-delete-failed = Failed to delete file
file-too-large = File is too large
file-invalid-type = Invalid file type
file-name-too-long = File name is too long
file-alt-text-too-long = Alt text is too long

# Drive
drive-capacity-exceeded = Drive capacity exceeded
drive-file-not-found = File not found

# Admin/Instance
admin-user-not-found = User not found
admin-cannot-modify-admin = Cannot modify admin
admin-cannot-modify-self = Cannot modify yourself
admin-suspend-success = User suspended
admin-unsuspend-success = User unsuspended
admin-delete-success = User deleted
admin-password-reset-success = Password reset
admin-email-sent = Email sent

# Federation
federation-instance-not-found = Instance not found
federation-instance-blocked = Instance blocked
federation-instance-silenced = Instance silenced
federation-instance-suspended = Instance suspended
federation-relay-added = Relay added
federation-relay-removed = Relay removed
federation-relay-not-found = Relay not found
federation-inbox-url-required = Inbox URL is required
federation-invalid-inbox-url = Invalid inbox URL

# Import/Export
export-in-progress = Export in progress
export-completed = Export completed
export-failed = Export failed
import-in-progress = Import in progress
import-completed = Import completed
import-failed = Import failed

# Search
search-no-results = No results found
search-invalid-query = Invalid search query
search-rate-limited = Search rate limited

# Lists
list-not-found = List not found
list-name-too-long = List name is too long
list-too-many = Too many lists
list-user-already-added = User already in list
list-user-not-found-in-list = User not found in list

# Groups
group-not-found = Group not found
group-name-too-long = Group name is too long
group-description-too-long = Group description is too long
group-member-not-found = Member not found
group-already-member = Already a member
group-not-member = Not a member
group-invitation-expired = Invitation expired

# Antennas
antenna-not-found = Antenna not found
antenna-name-too-long = Antenna name is too long
antenna-keywords-too-many = Too many keywords
antenna-keywords-too-long = Keywords too long

# Clip
clip-not-found = Clip not found
clip-name-too-long = Clip name is too long
clip-description-too-long = Clip description is too long
clip-too-many = Too many clips
note-already-clipped = Post already clipped

# Generic
yes = Yes
no = No
ok = OK
cancel = Cancel
save = Save
delete = Delete
edit = Edit
close = Close
loading = Loading...
load-more = Load more
error = Error
success = Success
warning = Warning
info = Information
confirm = Confirm
done = Done
back = Back
next = Next
previous = Previous
refresh = Refresh
search = Search
create = Create
add = Add
remove = Remove
update = Update
copy = Copy
paste = Paste
cut = Cut
select = Select
select-all = Select all
undo = Undo
redo = Redo
share = Share
report = Report
block = Block
unblock = Unblock
mute = Mute
unmute = Unmute
hide = Hide
show = Show
pin = Pin
unpin = Unpin
follow = Follow
unfollow = Unfollow
request = Request
accept = Accept
reject = Reject
welcome = Welcome
farewell = Goodbye

# Server info
server-info = Server Information
server-name = Server Name
server-description = Server Description
server-admin = Server Admin
server-rules = Server Rules
tos = Terms of Service
privacy-policy = Privacy Policy

# HTTP status
http-400 = Bad Request
http-401 = Unauthorized
http-403 = Forbidden
http-404 = Not Found
http-429 = Too Many Requests
http-500 = Internal Server Error
http-502 = Bad Gateway
http-503 = Service Unavailable
