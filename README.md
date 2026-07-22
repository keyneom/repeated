# repeated

Procedural macro crate (syn 2 / edition 2021) that repeats a code template over a
**numeric range** or an **enumerated list** of tokens.

Add to `Cargo.toml`:

```toml
[dependencies]
repeated = "0.1"
```

## Ranges

`for <id> in [low; high]` repeats inclusive with step `1`. Optional third
component: `[low; high; step]`.

Placeholders use `%%id%%`. To concatenate onto an identifier, end the prefix with `_` (for example `name_%%id%%` → `name_0`). A bare keyword before a placeholder is left alone (`for %%T%%` stays `for u8`).

```rust
use repeated::repeated;

fn main() {
    let mut v = Vec::<u32>::with_capacity(256);
    repeated!(for z in [0; 255] {
        v.push(%%z%%);
    });
    assert_eq!(v.len(), 256);

    // Empty separator glues repetitions (useful for digit concatenation).
    let n = repeated!(for _unused in [0; 3] { 1 }, "");
    assert_eq!(n, 1111);

    repeated!(for i in [0; 4; 2] {
        println!("%%i%%");
    });
}
```

Nested macros work because the outer expansion substitutes before the inner
`repeated!` runs:

```rust
use repeated::repeated;

repeated!(for x in [0; 9; 3] {
    fn welcome_%%x%%() {
        repeated!(for y in [1; %%x%%; 2] {
            println!("From within the macro %%x%%:%%y%%!");
        });
    }
});
```

## Enumerated lists

Comma-separated lists substitute each token (idents or literals). Handy for
duplicate `impl`s:

```rust
use repeated::repeated;

trait Desc {
    fn desc() -> &'static str;
}

repeated!(for T in [u8, u16, u32] {
    impl Desc for %%T%% {
        fn desc() -> &'static str {
            stringify!(%%T%%)
        }
    }
});

assert_eq!(<u8 as Desc>::desc(), "u8");
```

Non-contiguous numeric lists and plain idents work the same way:

```rust
use repeated::repeated;

let mut values = Vec::new();
repeated!(for i in [0, 2, 4, 6] {
    values.push(%%i%%);
});
assert_eq!(values, [0, 2, 4, 6]);

repeated!(for name in [frank, bill, ted] {
    fn hello_%%name%%() {}
});
```

Use `;` for ranges and `,` for lists. `[0]` is a one-element list; `[0; 1]` is a
two-endpoint range.

## Prelude / postlude

Some positions (for example match arms) are awkward for a bare macro invocation.
Wrap unbalanced tokens with named prelude/postlude markers:

```rust
use repeated::repeated;

fn main() {
    let t = 3;
    repeated!(
        %% my prelude
        match t {
        prelude my %%
        for x in [0; 15] {
            %%x%% => {
                println!("{}", repeated!(for y in [0; %%x%%] { %%x%% }, ""));
            }
        }
        %% my postlude
        _ => panic!(),
        }
        postlude my %%
    );
}
```

## Performance

Large expansions used to be dominated by string-joining the whole output and
re-lexing it. The default path now:

1. Compiles the repeat body into a template once (placeholder scan once).
2. Instantiates per value with token cloning.
3. Assembles via flat tokens / `unflatten` when the separator is whitespace
   (including the default newline), preserving spans.

Empty-string separators still use a text glue path so `1`×4 can become `1111`.

For huge bodies, prefer many small items (separate functions) over one giant
function. Example pattern:

```rust
use repeated::repeated;

repeated!(
    for j in [0; 255] {
        fn repeat_%%j%%() {
            repeated!(for i in [0; %%j%%] { println!("%%i%%"); });
        }
    }
);
```

Stress compile locally:

```text
cargo clean -p repeated
time cargo build --example stress_match
time cargo test --test compile_stress
```

Recorded numbers live in [`docs/COMPILE_TIMES.md`](docs/COMPILE_TIMES.md).

## Errors and spans

Parse errors (bad ranges, unclosed prelude markers, …) use `syn::Error` with
spans on the offending tokens — including when a prelude/postlude wraps the
body. Expanded tokens keep template spans on the fast path so rustc type errors
point into the macro input rather than only at the invocation site.

## License

MIT
