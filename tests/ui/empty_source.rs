use repeated::repeated;

fn main() {
    // Empty brackets — neither a range nor an enumerated list.
    repeated!(for x in [] {
        let _ = %%x%%;
    });
}
