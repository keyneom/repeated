use repeated::repeated;

/// Focused expansion coverage beyond the broader smoke tests.
#[test]
fn concatenates_idents_around_placeholders() {
    repeated!(for n in [0; 2] {
        fn item_%%n%%() -> u32 { %%n%% }
    });
    assert_eq!(item_0(), 0);
    assert_eq!(item_1(), 1);
    assert_eq!(item_2(), 2);
}

#[test]
fn substitutes_placeholders_inside_string_literals() {
    let mut messages = Vec::new();
    repeated!(for n in [1; 2] {
        messages.push("value=%%n%%");
    });
    assert_eq!(messages, ["value=1", "value=2"]);
}

#[test]
fn substitutes_nested_placeholders_independently() {
    let mut pairs = Vec::new();
    repeated!(for x in [0; 1] {
        repeated!(for y in [2; 3] {
            pairs.push((%%x%%, %%y%%));
        });
    });
    assert_eq!(pairs, [(0, 2), (0, 3), (1, 2), (1, 3)]);
}

#[test]
fn supports_stepped_ranges() {
    let mut values = Vec::new();
    repeated!(for i in [0; 6; 2] {
        values.push(%%i%%);
    });
    assert_eq!(values, [0, 2, 4, 6]);
}

#[test]
fn supports_enumerated_type_lists() {
    trait SizeOf {
        fn size() -> usize;
    }

    repeated!(for T in [u8, u16, u32] {
        impl SizeOf for %%T%% {
            fn size() -> usize {
                core::mem::size_of::<%%T%%>()
            }
        }
    });

    assert_eq!(<u8 as SizeOf>::size(), 1);
    assert_eq!(<u16 as SizeOf>::size(), 2);
    assert_eq!(<u32 as SizeOf>::size(), 4);
}

#[test]
fn supports_enumerated_numeric_lists() {
    let mut values = Vec::new();
    repeated!(for i in [0, 2, 4, 6] {
        values.push(%%i%%);
    });
    assert_eq!(values, [0, 2, 4, 6]);
}

#[test]
fn supports_enumerated_ident_concatenation() {
    repeated!(for name in [alpha, beta] {
        fn greet_%%name%%() -> &'static str {
            "%%name%%"
        }
    });
    assert_eq!(greet_alpha(), "alpha");
    assert_eq!(greet_beta(), "beta");
}
