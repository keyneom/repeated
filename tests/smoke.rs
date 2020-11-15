use repeated::repeated;

#[test]
fn works() {
    println!("Tested repeated!");
    repeated!(for x in [0;9;3] {
        pub fn Welcome_%%x%%() {
            repeated!(for y in [1;%%x%%;2] {
                println!("From within the macro %%y%%!");
            });
        }
    });
    Welcome_3();
}