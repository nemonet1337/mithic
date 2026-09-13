# Fix Backend Build Errors Plan

## Problem Summary

The Docker build fails with two main errors in `backend/src/auth.rs`:

1. **Unresolved imports** (E0432): `SaltString` and `rand_core::OsRng` not found in `argon2::password_hash`
2. **Wrong method signature** (E0061): `hash_password` called with 2 arguments but takes 1
3. **Deprecation warnings** from `totp-rs` v6: `TOTP` renamed to `Totp`, deprecated methods

## Root Cause

- `argon2 = "0.6"` in workspace Cargo.toml uses `password-hash` v0.6 which has different API
- `SaltString` moved to `password_hash::SaltString` (not `argon2::password_hash::SaltString`)
- `rand_core` feature must be enabled on `password-hash` crate
- `hash_password` signature changed: now takes only password, generates salt internally
- `totp-rs` v6 renamed `TOTP` → `Totp` and deprecated old constructors

## Fix Plan

### 1. Update Workspace Dependencies (Cargo.toml)

**File:** `Cargo.toml` (workspace root)

- Add `rand_core` feature to `argon2` dependency (which transitively enables it on `password-hash`)
- Or add explicit `password-hash` dependency with `rand_core` feature

```toml
argon2 = { version = "0.6", features = ["rand_core"] }
```

### 2. Fix Imports in auth.rs

**File:** `backend/src/auth.rs`

Change imports from:
```rust
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
```

To:
```rust
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::{PasswordHash, SaltString, rand_core::OsRng};
```

### 3. Fix hash_password Function

**File:** `backend/src/auth.rs` (lines 60-68)

Current (broken):
```rust
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)  // ERROR: 2 args
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();
    Ok(password_hash)
}
```

Fixed:
```rust
pub fn hash_password(password: &str) -> Result<String> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes())  // FIXED: 1 arg, salt generated internally
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();
    Ok(password_hash)
}
```

### 4. Fix TOTP Deprecations

**File:** `backend/src/auth.rs`

- Replace `TOTP` with `Totp` (lines 8, 20, 21)
- Replace `TOTP::new(...)` with `Totp::builder()...build()` pattern
- Replace `secret.to_bytes()` with `secret.as_bytes()`
- Replace `totp.get_secret_base32()` with `totp.secret().to_base32()`
- Replace `totp.get_url()` with `totp.to_url()`

### 5. Fix PasswordHash Deprecation

**File:** `backend/src/auth.rs` (line 71)

Change:
```rust
let parsed_hash = PasswordHash::new(password_hash)
```

To (using PHC string parsing):
```rust
let parsed_hash = PasswordHash::new(password_hash)  // This is fine, just deprecated alias
// Better: use password_hash::phc::PasswordHash if needed
```

Actually, the deprecation says to import as `password_hash::phc::PasswordHash`, but `PasswordHash::new` still works. Can leave as-is or update import.

### 6. Verify Build

Run locally to confirm fixes:
```bash
cd backend && cargo build --release
```

## Validation Steps

1. Local cargo build succeeds
2. Docker build succeeds
3. No new warnings introduced
4. Password hashing/verification still works (manual test if needed)

## Out of Scope

- Upgrading argon2 to newer major version
- Changing password hashing algorithm
- Adding tests (existing tests should cover)

## Risks

- Low: API changes are mechanical, behavior unchanged
- Password hashing uses internally generated salt (secure default)
- TOTP changes are API-only, logic identical