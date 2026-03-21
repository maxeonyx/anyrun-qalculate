# Task: anyrun-qalculate

## Goal

Build an anyrun launcher plugin that provides qalculate-powered calculations via direct C++ FFI to libqalculate. This replaces the built-in rink plugin which has poor parsing (e.g. currency conversion only works with uppercase codes before the number).

## Why

The user wants a launcher calculator that works like qalculate — case-insensitive, flexible word order, unit conversions, currency with live rates, natural language math like `20% of 500`. The built-in rink plugin is too fragile (e.g. `1 usd in nzd` shows nothing, only `USD 1 in NZD` works).

## Architecture

- **Anyrun plugin** — a Rust `cdylib` crate that implements the anyrun plugin interface (4 functions: init, info, get_matches, handler)
- **C++ FFI** — a thin C++ wrapper (~50 lines) with `extern "C"` linkage, compiled via `cc` build script, calling libqalculate's `Calculator` class
- **No shelling out** — do NOT call the `qalc` CLI. Link directly to libqalculate for sub-millisecond response times
- **No prefix** — calculator results appear alongside app results (like the current rink plugin). No `=` prefix required.
- **Handler** — selecting a result copies it to clipboard (`HandleResult::Copy`)

### Anyrun plugin API

Plugins are shared libraries using `abi_stable`. The `anyrun-plugin` crate (from the anyrun repo) provides macros:

```rust
#[init] fn init(config_dir: RString) -> State { ... }
#[info] fn info() -> PluginInfo { ... }
#[get_matches] fn get_matches(input: RString, state: &mut State) -> RVec<Match> { ... }
#[handler] fn handler(selection: Match, state: &mut State) -> HandleResult { ... }
```

Reference implementation: the rink plugin at https://github.com/anyrun-org/anyrun/tree/master/plugins/rink (~100 LOC). Study it.

### C++ FFI layer

The C++ wrapper needs to:
1. Create a `Calculator` instance and call `loadGlobalDefinitions()` (loads units, currencies, constants)
2. Expose `calculate_string(expression) -> string` via `extern "C"`
3. Handle thread safety — anyrun calls `init` on one thread and `get_matches` on another. The `Calculator` object must be `Send`.
4. Use `pkg-config` in `build.rs` to find libqalculate headers and link flags

libqalculate C++ API basics:
```cpp
#include <libqalculate/Calculator.h>
Calculator *calc = new Calculator();
calc->loadGlobalDefinitions();
EvaluationOptions eo;
PrintOptions po;
po.number_fraction_format = FRACTION_DECIMAL;
MathStructure result = calc->calculate(expression, eo);
result.format(po);
string output = result.print(po);
```

### Build dependencies

- **Runtime:** `libqalculate` (Arch package: `libqalculate`, provides `libqalculate.so` + `qalc` CLI)
- **Build:** `libqalculate` (headers at `/usr/include/libqalculate/`), C++ compiler, `pkg-config`
- **Rust:** `anyrun-plugin` (from anyrun git repo), `abi_stable`, `cc` (build script), `serde` + `ron` (config)

## Environment

- **Machine:** CachyOS (Arch-based), Hyprland, x86_64
- **Rust:** 1.94.0
- **libqalculate:** 5.9.0 available in repos (not yet installed — `pacman -S libqalculate`)
- **anyrun:** 25.12.0, installed from AUR. Plugins live at `~/.config/anyrun/plugins/` or `/usr/lib/anyrun/`
- **Shell:** fish
- **tmux-bridge session:** `tb-ati` — use for sudo commands. Load the `tmux-bridge` skill.
- **GitHub:** `maxeonyx/anyrun-qalculate` repo already created. Remote is `origin`.

### Current anyrun config (`~/.config/anyrun/config.ron`)

```ron
Config(
  x: Fraction(0.5),
  y: Fraction(0.35),
  width: Absolute(800),
  height: Absolute(1),
  hide_icons: false,
  ignore_exclusive_zones: false,
  layer: Overlay,
  hide_plugin_info: true,
  close_on_click: true,
  show_results_immediately: false,
  max_entries: Some(8),
  plugins: [
    "libapplications.so",
    "librink.so",
  ],
)
```

The plugin will replace `librink.so` in this config once working.

## Done when

1. **Repo is set up** with TDD ratchet (load the `tdd-ratchet` skill), CI, AGENTS.md
2. **Black-box tests exist** that verify:
   - Latency: a calculation completes in under 50ms (the whole point of FFI over shelling out)
   - Parsing quality: `1 usd in nzd`, `1 USD in NZD`, `USD 1 in NZD`, `20% of 500`, `5 kg to lbs`, basic arithmetic — all return results
   - Clean interaction: the plugin .so loads without error, doesn't panic, handles empty/garbage input gracefully
   - Load the `verifying-work` skill — plan verification before implementation
3. **CI publishes an AUR package** with correct dependencies (`libqalculate` as runtime dep, `anyrun` as runtime dep)
4. **The AUR package is installed on this machine** and the calculator is working in anyrun (rink replaced)

## Process

- Load skills: `tdd-ratchet`, `verifying-work`, `code-principles`, `tmux-bridge`, `tools`
- This is a NEW project — set TDD ratchet baseline to initial commit
- Write failing tests FIRST, commit them as pending, then implement
- Commit and push frequently
- Use `tb-ati` tmux bridge session for any sudo operations (installing packages, etc.)
- When installing the AUR package locally, use `makepkg -si` or `paru` — ask the user to handle any interactive prompts via the tmux bridge

## Non-goals

- Don't build a config UI or complex configuration. A simple optional `qalculate.ron` with a prefix field (defaulting to empty string) is fine.
- Don't support pango markup initially
- Don't worry about debouncing — libqalculate FFI should be fast enough that per-keystroke calls are fine
