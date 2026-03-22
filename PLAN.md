# Plan

## Phases

### Phase 1: Dev infrastructure ✅
- Cargo.toml with deps (anyrun-plugin, abi_stable, cc, serde, ron, pkg-config)
- build.rs compiling C++ FFI wrapper
- TDD ratchet baseline
- CI (GitHub Actions) on Arch container
- Project compiles as cdylib

### Phase 2: C++ FFI layer ✅
- Real libqalculate bridge with exception safety across FFI boundary
- Rust FFI bindings with explicit error handling (`Result`, not `Option`)
- Persistent Calculator instance behind `Mutex` for thread safety

### Phase 3: Plugin implementation ✅
- init, info, get_matches, handler functions
- Thread safety (Calculator created in init, Mutex-wrapped, used in get_matches)
- Config (optional prefix in qalculate.ron)
- Input normalization: `in` → `to`, `% of` → `%*`
- Garbage input filtering via `looks_like_calculation_input` heuristic

### Phase 4: Tests & quality ✅
- 10 tests passing via `cargo ratchet`
- Latency: under 50ms for hot calculations
- Parsing: `1 USD in NZD`, `20% of 500`, `5 kg to lbs`, basic arithmetic
- Graceful handling of empty/garbage input
- TDD ratchet green

### Phase 5: AUR package ✅
- PKGBUILD with `options=('!lto')` (critical — makepkg LTO breaks C++ FFI linking)
- Published to AUR as `anyrun-qalculate-git`
- CI workflow for AUR publishing (needs `AUR_SSH_KEY` GitHub secret)

### Phase 6: Install & verify ✅
- Plugin installed at `/usr/lib/anyrun/libanyrun_qalculate.so`
- Replaced `librink.so` in anyrun config
- User confirmed working!

## Future improvements
- Lower-case currency support: `1 usd in nzd` (currently works via normalization, but worth verifying edge cases)
- Consider `USD 1 in NZD` word order (the rink failure that motivated this project)
- Pango markup support (non-goal for now per TASK-build.ignore.md)
- Debouncing (non-goal — FFI is fast enough per TASK-build.ignore.md)

## Lessons learned

- **Ratchet violations: rebase, don't re-baseline.** When a test rename caused a TDD ratchet violation, the subagent worked around it by re-baselining (moving the baseline commit forward). This is wrong — it defeats the purpose of the ratchet and pollutes history with workaround commits. The correct fix is always to rebase and split/fix the offending commit. This repo is single-developer, rebasing main is fine.
- **makepkg LTO breaks C++ FFI cdylibs.** Arch's default makepkg.conf enables LTO, which strips the statically-linked C++ bridge symbols from the final .so. Fix: `options=('!lto')` in PKGBUILD.
- **tmux bridge sessions expire.** Don't hardcode session names in AGENTS.md — they're ephemeral.
