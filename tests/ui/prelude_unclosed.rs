use repeated::repeated;

fn main() {
    // Opening prelude marker without a matching closer.
    repeated!(
        %% broken prelude
        let _x = 1;
        for y in [0; 1] {
            let _ = %%y%%;
        }
    );
}
