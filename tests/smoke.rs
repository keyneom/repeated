use repeated::repeated;

#[test]
fn works_nested() {
    repeated!(for x in [0;9;3] {
        fn Welcome_%%x%%() {
            repeated!(for y in [1;%%x%%;2] {
                println!("From within the macro %%x%%:%%y%%!");
            });
        }
    });
    Welcome_3();
    println!("Tested repeated!");
}

#[test]
fn works_with_match() {
    let t = 3;
    repeated!(
        %% my prelude
        match t {
        prelude my %%
        for x in [0;15] {
            %%x%% => {
                println!("{}", repeated!(for y in [0;%%x%%] {%%x%%},""));
            }
        }
        %% my postlude
        _ => panic!(),
        }
        postlude my %%
    );
}

#[test]
fn works_with_large_match() {
    let t = 3;
    repeated!(
        %% my prelude
        match t {
        prelude my %%
        for x in [0;255] {
            %%x%% => {
                println!("{}", 5+3+12+9);
            }
        }
        %% my postlude
        _ => panic!(),
        }
        postlude my %%
    );
}


#[test]
fn works_regardless_of_token_string_whitespacing() {
    let t = 3;
    repeated!(
        %% my prelude
        match t {
        prelude my %%
        for x in [0;15] {
            %%x%% => {
                println!("{}", 5);
            }
        }
        %% my postlude
        _ => panic!(),
        }
        postlude my %%
    );
}


#[test]
fn works_with_references_to_external_variables() {
    let mut v = Vec::<u32>::with_capacity(256);
    repeated!(for z in [0;255] {
        v.push(%%z%%);
    });
    println!("{:?}", v);
}

#[test]
fn works_to_create_values_on_a_single_line() {
    let n = repeated!(for _unused in [0;3] {1}, "");
    repeated!(for i in [0;4;2] { println!("%%i%%"); }, "");
    println!("Tested match position! Here is n: {}", n);
}
