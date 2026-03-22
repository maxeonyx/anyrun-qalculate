# Agent Instructions

## Project

Anyrun plugin for qalculate — a Rust `cdylib` that links to libqalculate via C++ FFI.

The current bridge uses the real libqalculate API, not a stub. Tests should exercise plugin-facing behavior against the real native library.

## Development

- **TDD ratchet enforced** — load the `tdd-ratchet` skill. New tests must fail first.
- **Verification first** — load the `verifying-work` skill. Plan how you'll verify before implementing.
- **Code principles** — load the `code-principles` skill. Fail fast, side effects at edges.
- **Shell** is fish. Use `$status` not `$?` for exit codes in tb commands.
- **tmux-bridge** — start a session for sudo operations. Load the `tmux-bridge` skill.

## Build & Test

```bash
pacman -S libqalculate pkgconf gcc   # runtime + build deps (headers, pkg-config, C++ compiler)
cargo build --release     # produces target/release/libanyrun_qalculate.so
cargo ratchet             # canonical test command; cargo test is gatekept
```

## Packaging

- AUR package name: `anyrun-qalculate-git`
- `makepkg` on Arch enables LTO by default; this package must keep `options=('!lto')` in `PKGBUILD` or the resulting plugin can drop its `libqalculate`/`libstdc++` dynamic links
- For this `-git` package, generate `.SRCINFO` only after running `makepkg --nobuild --nodeps` (or equivalent) so `pkgver()` has run first
- Packaging smoke verification should confirm `readelf -d` on the packaged `/usr/lib/anyrun/libanyrun_qalculate.so` includes `libqalculate.so.23` and `libstdc++.so.6`

## Task file

Read `TASK-build.ignore.md` for the full task specification.
