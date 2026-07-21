use repeated::repeated;

fn main() {
    // Too many range components — should produce a syn::Error, not a panic.
    repeated!(for x in [0; 1; 2; 3] {
        let _ = %%x%%;
    });
}
