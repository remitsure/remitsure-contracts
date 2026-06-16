# Contributing to remitsure-contracts

Thanks for helping out. RemitSure takes part in [Drips Wave](https://www.drips.network/wave), so many issues are scoped for Wave contributors.

## Picking up an issue

1. Look for issues labeled `wave` or `good first issue`.
2. Apply through the Wave app and wait to be assigned before you start.
3. Ask in the issue thread if anything is unclear. Small questions are welcome.

## Setup

You need Rust (stable), the `wasm32v1-none` target and the [Stellar CLI](https://developers.stellar.org/docs/tools/cli).

```bash
rustup target add wasm32v1-none
make check   # fmt, clippy, tests, wasm build
```

## Rules of thumb

- Every behaviour change needs a test in `contracts/remittance/src/test.rs`.
- Changing an event in `events.rs` breaks the backend indexer in remitsure-backend. Call it out in the PR.
- Keep PRs to one issue and reference it with `Closes #123`.
- CI must be green before review.
