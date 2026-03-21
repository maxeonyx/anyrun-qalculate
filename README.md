# anyrun-qalculate

Anyrun plugin providing a qalculate-powered calculator via libqalculate C++ FFI.

This project follows strict TDD — see the `tdd-ratchet` skill. Load the `verifying-work` skill before implementing.

`cargo ratchet` is the canonical test command for this repo. `cargo test` is intentionally blocked by a gatekeeper test unless `TDD_RATCHET=1` is set by the ratchet.

## Build

Requires `libqalculate`, `pkgconf`, a C++ compiler, and Rust toolchain.

```bash
pacman -S libqalculate pkgconf gcc  # provides libqalculate.so, headers, pkg-config, and C++ compiler
cargo build --release
cargo ratchet
cp target/release/libanyrun_qalculate.so ~/.config/anyrun/plugins/
```

## Install from AUR

```bash
paru -S anyrun-qalculate
```
