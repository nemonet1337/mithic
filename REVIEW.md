# REVIEW.md

## What matters in this repository
- Preserve note visibility (`public` / `home` / `followers` / `specified`) and block/mute isolation on every query and ActivityPub delivery.
- Inbox requires HTTP Signature (RSA-SHA256) via `mithic_server::federation::http_sig`. Unsigned or mismatched signatures are high-risk.
- Treat JWT + Argon2 auth, token revocation, account deletion, and admin suspend/delete as high-risk changes.
- Keep the SSRF guard on server-side fetches; do not fetch private or reserved addresses.
- Do not add Misskey or Mastodon client-compat APIs. Federation is ActivityPub only; `/api/v1/*` is Mithic-native.
- Prefer small, explicit fixes. Crate layout stays `backend` / `shared` / `frontend`.

## Severity calibration
- Critical: private-note leakage, HTTP Signature bypass, JWT or secret exposure, SSRF to private networks, mass deletion.
- Warning: missing visibility checks, rate-limit gaps, untested ActivityPub activities, unsafe fetch defaults.
- Do not flag rustfmt or clippy nits when CI already enforces them.

## Verification expectations
- New API or federation rules need tests that assert the observable result (`http_sig`, ActivityPub, and query tests already exist).
- Run `cargo test -p mithic-server -p shared` for backend changes.
- UI changes (Leptos PWA) should preserve keyboard use, the mobile dock, and the offline page.
