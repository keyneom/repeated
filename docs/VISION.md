# repeated — product vision

## What we're building

A **Rust procedural macro** for repeating code templates at compile time — numeric ranges and **enumerated token lists** — with `%%placeholder%%` substitution, ident concatenation, and prelude/postlude blocks. It must be **practical on latest stable Rust**: fast enough expansion that real projects do not hit multi-minute compile times.

## User stories

1. **As a macro user**, I want `repeated!(for i in [0; 255] { ... })` without hand-copying hundreds of lines.
2. **As a generics author**, I want `for T in [u8, u16, u32] { impl ... for %%T%% { ... } }` — enumerated substitution, not just numeric loops.
3. **As a maintainer**, I want token-tree expansion (not string regex hacks) so placeholders work on modern `proc-macro2` and errors point at real spans.
4. **As a downstream crate**, I need **compile-time performance** measured and guarded — large expansions should stay in the low seconds, not unusable.

## Technical commitments

- **syn 2**, edition 2021, GitHub metadata on crates.io
- FlatToken assembly path; template compiled once per expansion site
- trybuild + expansion tests for regressions

## Compile-time performance

Methodology and benchmark results: [COMPILE_TIMES.md](COMPILE_TIMES.md)

Stress harness: `examples/stress_match.rs`, `tests/compile_stress.rs`
