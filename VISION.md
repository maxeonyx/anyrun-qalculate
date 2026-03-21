# Vision

## Why

The built-in anyrun rink calculator plugin has poor parsing — currency conversion only works with uppercase codes before the number (e.g. `USD 1 in NZD` works but `1 usd in nzd` doesn't). This is frustrating.

## What

An anyrun launcher plugin powered by libqalculate via direct C++ FFI. Qalculate handles case-insensitive input, flexible word order, unit conversions, live currency rates, and natural language math like `20% of 500`.

## Stakeholders

### End user (Max)

- Wants a launcher calculator that just works — type natural expressions and get results
- `1 usd in nzd`, `20% of 500`, `5 kg to lbs`, basic arithmetic should all return results
- No prefix required — calculator results appear alongside app results
- Sub-millisecond response time (FFI, not shelling out to CLI)
- Selecting a result copies it to clipboard

### Developer/maintainer (Max)

- Clean Rust crate with TDD ratchet enforced
- CI on GitHub
- Sustainable to maintain — thin C++ FFI layer, minimal config
- AUR package for easy install

### Future AUR users

- `paru -S anyrun-qalculate` just works
- Runtime deps: `libqalculate`, `anyrun`
