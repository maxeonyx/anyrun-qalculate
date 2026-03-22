# anyrun-qalculate

Anyrun plugin providing a qalculate-powered calculator via libqalculate C++ FFI.

This project follows strict TDD — see the `tdd-ratchet` skill. Load the `verifying-work` skill before implementing.

`cargo ratchet` is the canonical test command for this repo. `cargo test` is intentionally blocked by a gatekeeper test unless `TDD_RATCHET=1` is set by the ratchet.

The current plugin uses the real libqalculate C++ API through a thin native bridge. Tests exercise the plugin-facing match flow against the real library, including arithmetic, unit conversion, currency conversion, percentage input, garbage filtering, and hot-path latency.

Input is currently normalized slightly before evaluation to better match user expectations with libqalculate 5.9.0: `in` is translated to `to` for conversions, and `% of` is translated to `%*`.

CI runs in an Arch Linux container so the native libqalculate headers and linker behavior stay close to the local CachyOS development environment.

## Build

Requires `libqalculate`, `pkgconf`, a C++ compiler, and Rust toolchain.

```bash
pacman -S libqalculate pkgconf gcc  # provides libqalculate.so, headers, pkg-config, and C++ compiler
cargo build --release
cargo ratchet
sudo install -Dm755 target/release/libanyrun_qalculate.so /usr/lib/anyrun/libanyrun_qalculate.so
```

## CI

GitHub Actions mirrors the local feedback loop:

```bash
cargo build --release
cargo ratchet
makepkg -f
```


## Install from AUR

```bash
paru -S anyrun-qalculate-git
```

The package installs `libanyrun_qalculate.so` to `/usr/lib/anyrun/`, alongside anyrun's bundled plugins.

To manually update the AUR package metadata for this `-git` package, regenerate `.SRCINFO` after `makepkg --nobuild --nodeps`, then push only `PKGBUILD` and `.SRCINFO` to `ssh://aur@aur.archlinux.org/anyrun-qalculate-git.git`.
