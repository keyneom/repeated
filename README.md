# repeated #

This is a procedural macro that allows you to repeat a template of code a number of times.

## Examples ##

From the tests:

This will push each value from 0 to 255 (inclusive) to the vec v.
```rust
let mut v = Vec::<u32>::with_capacity(256);
repeated!(for z in [0;255] {
    v.push(%%z%%);
});
println!("{:?}", v);
```

After defining your for loop to describe your repeated block you can pass a string literal to describe how each repitition should be joined to the prior iteration. Here we use an empty string literal to keep each repitition next to each other.
We first create an int `n` with a value of `111` and continue on to define another repeated block. The second repeated block has an option third parameter on the range definition indicating the step by which we should be incrementing.
```rust
let n = repeated!(for _unused in [0;3] {1}, "");
repeated!(for i in [0;4;2] { println!("%%i%%"); }, "");
println!("Tested match position! Here is n: {}", n);
```
The previous block will print the following:
```
0
2
4
Tested match position! Here is n: 1111
```

The following example demonstrates nested usage where a block is repeated a variable number of times based off of the current iteration of the outer block:
```rust
repeated!(for x in [0;9;3] {
    fn Welcome_%%x%%() {
        repeated!(for y in [1;%%x%%;2] {
            println!("From within the macro %%x%%:%%y%%!");
        });
    }
});
Welcome_3();
```

Finally, there are certain scenarios where it isn't valid to make a call to a macro. One such scenario is in a match branch position.
In the example below you can see how you can continue to make use of the repeated macro even when attempting to create similar match branches.
You can define a `prelude` or a `postlude` to your main for body. In order to do so, you must provide an identifier for your pre/post-lude.
The example below provides an example of defining a prelude and postlude that each use the identifier of `my`.
```
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
```
This will print `3333` to the console.

### Contribution guidelines ###

Tests and a warning/error free build.

I'd like to improve error handling in the case you have a prelude or postlude defined to match the error handling you get without them.

Another nice to have would be to support iteration over something other than integers.