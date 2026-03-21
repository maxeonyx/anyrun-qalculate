# anyrun-qalculate

Anyrun plugin providing a qalculate-powered calculator via libqalculate C++ FFI.

This project follows strict TDD — see the `tdd-ratchet` skill. Load the `verifying-work` skill before implementing.

## Build

Requires `libqalculate` (system package) and Rust toolchain.

```bash
pacman -S libqalculate  # provides qalc CLI + libqalculate.so + headers
cargo build --release
cp target/release/libanyrun_qalculate.so ~/.config/anyrun/plugins/
```

## Install from AUR

```bash
paru -S anyrun-qalculate
```
