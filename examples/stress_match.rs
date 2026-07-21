//! Compile-time stress example for `repeated!`.
//!
//! Builds a 5000-arm match (prelude/postlude) plus 5000 sequential statements.
//! This exercises the fast FlatToken assembly path used for the default
//! newline separator (no string round-trip / re-lex of the full expansion).
//!
//! Measure expansion-dominated compile time with:
//!
//! ```text
//! cargo clean -p repeated
//! time cargo build --example stress_match
//! ```
//!
//! On a warm dependency cache, expect on the order of ~1s or less for this
//! example after the token-assembly expansion path (vs multi-second to much
//! longer historically when the whole expansion was joined as a String and
//! re-parsed).

use repeated::repeated;

fn main() {
    let x = 42u32;
    repeated!(
        %%s prelude
        match x {
        prelude s%%
        for j in [0; 4999] {
            %%j%% => { let _v = %%j%%; }
        }
        %%e postlude
            _ => {}
        }
        postlude e%%
    );

    let mut n = 0u32;
    repeated!(for i in [0; 4999] {
        n = n.wrapping_add(%%i%%);
    });
    let _ = n;
}