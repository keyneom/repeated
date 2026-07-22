use repeated::repeated;

fn main() {
    // Expanded tokens keep the template span so rustc points into the macro input.
    repeated!(
        %% my prelude
        fn check() {
        prelude my %%
        for _n in [0; 0] {
            let _: u8 = "not a u8";
        }
        %% my postlude
        }
        postlude my %%
    );
}
