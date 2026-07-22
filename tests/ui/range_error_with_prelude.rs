use repeated::repeated;

fn main() {
    // Prelude/postlude must not erase the span on the invalid range.
    repeated!(
        %% my prelude
        let _ = {
        prelude my %%
        for x in [0; 1; 2; 3] {
            let _ = %%x%%;
        }
        %% my postlude
        };
        postlude my %%
    );
}
