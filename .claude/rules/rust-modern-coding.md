# Rust Modern Coding Standards

## General Principles
- Write idiomatic Rust. Prefer clarity and safety over micro-optimizations unless profiling indicates a need.
- Leverage the type system to make illegal states unrepresentable.
- Keep functions focused. Prefer small, composable units.
- Avoid unnecessary allocations and clones. Prefer borrowing when possible.

## Ownership, Borrowing & Lifetimes
- Prefer borrowing (`&T`, `&mut T`) over owning when the data does not need to be moved.
- Use lifetime elision whenever the compiler can infer lifetimes correctly.
- Avoid `'static` bounds unless truly required.
- Prefer `Cow<'_, T>` when both owned and borrowed data are common.

## Error Handling
- Prefer `Result<T, E>` and `Option<T>` over panics for recoverable errors.
- Use `thiserror` for library error types and `anyhow` for application-level error handling when appropriate.
- Propagate errors with `?` rather than manually matching in most cases.
- Provide context when converting errors (e.g., `.context("...")` with anyhow).

## Type System & Abstractions Design
- Prefer enums + pattern matching over boolean flags or complex inheritance-like structures.
- Use newtype pattern to add semantic meaning and enforce invariants.
- Prefer trait objects (`dyn Trait`) only when dynamic dispatch is genuinely needed; static dispatch is preferred otherwise.
- Avoid over-abstraction. Do not introduce traits or generics that are used only once.

## Performance & Safety
- Run `clippy` and address its suggestions unless there is a clear reason not to.
- Prefer `Iterator` adapters over manual loops when they improve clarity.
- Use `#[must_use]` on important return values.
- Avoid `unsafe` unless there is a compelling, documented reason. Isolate unsafe blocks and document safety invariants.

## Module & API Design
- Keep modules cohesive. Prefer small, focused modules over large files.
- Make the public API as small as practical. Prefer `pub(crate)` or private visibility by default.
- Document public items with rustdoc comments.