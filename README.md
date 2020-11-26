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
We first create an int `n` with a value of `111` and continue on to define another repeated block. The second call to repeated has an optional third parameter defined on the range definition (the array looking syntax immediately following the `in` keyword) indicating the step by which we should be incrementing.
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
```rust
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

## Performance ##
Large repititions can become very slow to compile if you aren't careful. It can be beneficial to slowly increment the number of repititions to the number you would like instead of immediately setting the number of repititions to the final number immediately. If you find that compile times are being significantly impacted it might be useful to avoid large functions. As an example, the following code takes an extremely large amount of time to compile and might simply error out at some point:
```rust
repeated!(
    %%s prelude
    match x {
    prelude s%%
    for j in [0;255] {
        %%j%% => {
            repeated!(for i in [0;%%j%%] { println!("%%i%%"); });
        }
    }
    %%e postlude
        _ => {
            println!("No match was found!!");
        }
    }
    postlude e%%);
```
The above code creates a match statement that has more than 32,000 `println!` statements in it. Having all of this in a single function can slow down the compiler significantly. Instead we can refactor the code above to an example we have in the tests:
```rust
repeated!(
    for j in [0;255] {
        fn repeat_%%j%%() {
            repeated!(for i in [0;%%j%%] { println!("%%i%%"); });
        }
    }
);

#[test]
fn large_expansions_are_still_performant() {
    let x = 128;
    repeated!(
        %%s prelude
        match x {
        prelude s%%
        for j in [0;255] {
            %%j%% => {
                repeat_%%j%%();
            }
        }
        %%e postlude
            _ => {
                println!("No match was found!!");
            }
        }
        postlude e%%);
}
```
We still have a nested repeat statement but this time we are defining separate functions that execute the `println!`'s. The repeated match branches now no longer have hundreds of lines of code within them but instead only contain a simple call to one of the functions we've defined. This code compiles much faster than the previous example.

## Contribution guidelines ##

Tests and a warning/error free build.

I'd like to improve error handling in the case you have a prelude or postlude defined to match the error handling you get without them.

Generating the repitition blocks' tokenstreams is still quiet slow in some cases. I believe this is primarily a result of the string based parsing that is currently being used to create the repeated tokens. Switching to creating cloned tokenstreams within the repeated blocks would likely be faster and solve our current error reporting issues when a prelude or postlude is present.

Another nice to have would be to support iteration over an explicitly enumerated list of values. In essence, instead of using a `RepeatRange` definition of `[0;6;2]` to repeat with `0,2,4,6` you could pass a `[0,2,4,6]` to get the same effect. This provides more flexibility in that you can define values that are not simply incremented by some constant amount, but it can be much less friendly when needing to provide a large number of repititions. This can come in handy when creating duplicate impls for `u8`,`u16`,`u32`,`u64`, etc. In particular this is beneficial because it allows for the use of non-numeric identifiers, such as `[frank, bill, ted]`.