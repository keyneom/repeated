//! Compile-stress coverage for large `repeated!` expansions.
//!
//! Runtime is trivial; the value is ensuring a several-thousand-iteration
//! expansion keeps compiling quickly on stable Rust. Time with:
//! `time cargo test --test compile_stress`

use repeated::repeated;

#[test]
fn large_match_expands_without_string_roundtrip_path() {
    let x = 7u32;
    let hit;
    repeated!(
        %%s prelude
        match x {
        prelude s%%
        for j in [0; 1023] {
            %%j%% => { hit = %%j%%; }
        }
        %%e postlude
            _ => { hit = 9999; }
        }
        postlude e%%
    );
    assert_eq!(hit, 7);
}

#[test]
fn large_statement_list_expands() {
    let mut n = 0u32;
    repeated!(for i in [0; 1023] {
        n = n.wrapping_add(%%i%%);
    });
    // sum 0..=1023
    assert_eq!(n, 1023 * 1024 / 2);
}