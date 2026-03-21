# Plan

## Phases

### Phase 1: Dev infrastructure ← CURRENT
- Cargo.toml with deps (anyrun-plugin, abi_stable, cc, serde, ron, pkg-config)
- build.rs compiling C++ FFI wrapper
- TDD ratchet baseline
- CI (GitHub Actions)
- Project compiles (even if plugin functions are stubs)

### Phase 2: C++ FFI layer
- Thin C++ wrapper calling libqalculate Calculator class
- Rust FFI bindings
- Tests: can call calculate, returns results, handles errors

### Phase 3: Plugin implementation
- init, info, get_matches, handler functions
- Thread safety (Calculator created in init, used in get_matches on different thread)
- Config (optional prefix in qalculate.ron)

### Phase 4: Tests & quality
- Latency: calculation completes in under 50ms
- Parsing quality: `1 usd in nzd`, `1 USD in NZD`, `USD 1 in NZD`, `20% of 500`, `5 kg to lbs`, basic arithmetic
- Graceful handling of empty/garbage input
- TDD ratchet green

### Phase 5: AUR package
- PKGBUILD
- CI publishes to AUR
- Correct dependencies

### Phase 6: Install & verify
- Install on this machine via AUR package
- Replace rink in anyrun config
- Manual verification of all key expressions

## Notes

- Phases 2-4 will overlap — TDD means tests come before implementation
- Phase 1 is the foundation — don't rush it
