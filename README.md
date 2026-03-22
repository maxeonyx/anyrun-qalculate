# anyrun-qalculate

An anyrun calculator plugin powered by libqalculate via a native C++ FFI bridge.

`anyrun-qalculate` brings fast, flexible calculator results into anyrun without shelling out or requiring a prefix by default.

## What it does

- Natural expressions like `2 + 2`
- Case-insensitive currency conversion like `1 usd in nzd`
- Natural percentage input like `20% of 500`
- Unit conversion like `5 kg to lbs`
- Sub-millisecond hot-path calculations via direct libqalculate FFI

Example expressions:

```text
2 + 2
1 usd in nzd
20% of 500
5 kg to lbs
```

## Install from AUR

```bash
paru -S anyrun-qalculate-git
```

AUR package: https://aur.archlinux.org/packages/anyrun-qalculate-git

The package installs `libanyrun_qalculate.so` to `/usr/lib/anyrun/` alongside anyrun's other plugins.

## Configuration

Add the plugin to your anyrun config, typically `~/.config/anyrun/config.ron`:

```ron
Config(
  plugins: [
    "libapplications.so",
    "libanyrun_qalculate.so",
  ],
)
```

By default, calculations appear without a prefix. If you want one, create `~/.config/anyrun/qalculate.ron`:

```ron
(prefix: "=")
```

With that config, queries must start with `=` such as `= 2 + 2`.

## Build from source

Requires `libqalculate`, `pkgconf`, `gcc`, and a Rust toolchain.

```bash
pacman -S libqalculate pkgconf gcc
cargo build --release
cargo ratchet
sudo install -Dm755 target/release/libanyrun_qalculate.so /usr/lib/anyrun/libanyrun_qalculate.so
```

`cargo ratchet` is the canonical test command for this repo.

## Development notes

The plugin uses the real libqalculate API through a thin native bridge, with tests covering plugin-facing behavior against the real library. Input is lightly normalized so `in` maps to `to` for conversions and `% of` maps to `%*` for percentage expressions.
