# Compile-time measurements

Measurements on the machine that last updated this file. Re-run after hot-path
changes; numbers are wall-clock and include rustc work beyond the proc-macro.

## Environment

- Recorded: 2026-07-21
- `rustc`: rustc 1.95.0 (59807616e 2026-04-14)
- Host: Avnaim

## Stress example (`examples/stress_match.rs`)

Builds a 5000-arm match (prelude/postlude) plus 5000 sequential statements on
the FlatToken assembly path (default newline separator — no full-string re-lex).

```text
cargo clean -p repeated
time cargo build --example stress_match
```

| Run | Wall time | Notes |
|-----|-----------|-------|
| 1 | 2.109s | after `cargo clean -p repeated`; deps warm |

## Compile-stress tests

```text
cargo build   # warm the proc-macro crate
time cargo test --test compile_stress
```

| Run | Wall time | Notes |
|-----|-----------|-------|
| 1 | 1.435s | 2×1024-iteration expansions |

## Hot-path notes

- Repeat bodies are compiled once into a `Template` (placeholder positions
  resolved once), then instantiated per substitution value.
- Default / whitespace separators assemble via `FlatTokens` + `unflatten`
  (spans preserved; no `TokenStream` string round-trip).
- Empty-string separators still glue via text for digit/ident concatenation
  (`1`×4 → `1111`) and re-lex the result.
- Ident concatenation only applies when the prefix ident ends with `_`
  (`item_%%n%%`), so `for %%T%%` remains two tokens.
