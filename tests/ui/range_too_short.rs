use repeated::repeated;

fn main() {
    // Missing high bound — should produce a syn::Error, not a panic.
    repeated!(for x in [0] {
        let _ = %%x%%;
    });
}
