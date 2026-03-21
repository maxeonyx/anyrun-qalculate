# Agent Instructions

## Project

Anyrun plugin for qalculate — a Rust `cdylib` that links to libqalculate via C++ FFI.

## Development

- **TDD ratchet enforced** — load the `tdd-ratchet` skill. New tests must fail first.
- **Verification first** — load the `verifying-work` skill. Plan how you'll verify before implementing.
- **Code principles** — load the `code-principles` skill. Fail fast, side effects at edges.
- **Shell** is fish. Use `$status` not `$?` for exit codes in tb commands.
- **tmux-bridge** session `tb-ati` — use for sudo. Load the `tmux-bridge` skill.

## Build & Test

```bash
pacman -S libqalculate   # runtime + build dep (headers + .so)
cargo build --release     # produces target/release/libanyrun_qalculate.so
cargo test                # run tests
cargo ratchet             # run TDD ratchet
```

## Task file

Read `TASK-build.ignore.md` for the full task specification.
