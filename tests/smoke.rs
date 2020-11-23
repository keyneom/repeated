use std::{str::Chars, time::Instant};

use repeated::repeated;
#[macro_use]
extern crate lazy_static;
use regex::Regex;

// #[test]
// fn works_nested() {
//     repeated!(for x in [0;9;3] {
//         fn Welcome_%%x%%() {
//             repeated!(for y in [1;%%x%%;2] {
//                 println!("From within the macro %%x%%:%%y%%!");
//             });
//         }
//     });
//     Welcome_3();
//     println!("Tested repeated!");
// }

// #[test]
// fn works_with_match() {
//     let t = 3;
//     repeated!(
//         %% my prelude
//         match t {
//         prelude my %%
//         for x in [0;15] {
//             %%x%% => {
//                 println!("{}", repeated!(for y in [0;%%x%%] {%%x%%},""));
//             }
//         }
//         %% my postlude
//         _ => panic!(),
//         }
//         postlude my %%
//     );
// }

// #[test]
// fn works_with_large_match() {
//     let t = 3;
//     repeated!(
//         %% my prelude
//         match t {
//         prelude my %%
//         for x in [0;255] {
//             %%x%% => {
//                 println!("{}", 5+3+12+9);
//             }
//         }
//         %% my postlude
//         _ => panic!(),
//         }
//         postlude my %%
//     );
// }


// #[test]
// fn works_regardless_of_token_string_whitespacing() {
//     let t = 3;
//     repeated!(
//         %% my prelude
//         match t {
//         prelude my %%
//         for x in [0;15] {
//             %%x%% => {
//                 println!("{}", 5);
//             }
//         }
//         %% my postlude
//         _ => panic!(),
//         }
//         postlude my %%
//     );
// }


// #[test]
// fn works_with_references_to_external_variables() {
//     let mut v = Vec::<u32>::with_capacity(256);
//     repeated!(for z in [0;255] {
//         v.push(%%z%%);
//     });
//     println!("{:?}", v);
// }

// #[test]
// fn works_to_create_values_on_a_single_line() {
//     let n = repeated!(for _unused in [0;3] {1}, "");
//     repeated!(for i in [0;4;2] { println!("%%i%%"); }, "");
//     println!("Tested match position! Here is n: {}", n);
// }

// #[test]
// fn works_to_define_functions_with_x_params() {
//     repeated!(
//         %%s prelude let function: fn( prelude s%%
//         for j in [0;1] {f64,},""
//         %%e postlude ) -> f64 = |x,y| { x+y }; postlude e%%
//     );

//     function(4.0,5.0);

//     repeated!(
//     %% s prelude
//     function(
//     prelude s%%
//     for j in [0;1] {%%j%%.into(),}, ""
//     %%e postlude
//     );
//     postlude e%%);
// }

// #[test]
// fn works_to_define_variables() {
//     repeated!(for j in [0;1] {
//         let param_%%j%% =%%j%%;
//     });
//     println!("{}", param_0);
// }

repeated!(
    for j in [0;255] {
        fn repeat_%%j%%() {
            repeated!(
                %%s prelude
                // This is a test!
                prelude s%%
                for i in [0;%%j%%] { println!("%%i%%"); }
                %%e postlude
                // This is another test!
                postlude e%%
            );
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

// lazy_static! {
//     static ref PRELUDE_IDENT_RE: Regex =
//         Regex::new(r"^\s*?%\s*?%\s*?(?P<prelude_ident>\w*?)\s*?prelude\s").unwrap();
//     static ref POSTLUDE_IDENT_RE: Regex =
//         Regex::new(r"\spostlude\s*?(?P<postlude_ident>\w*?)\s*?%\s*?%\s*?$").unwrap();
// }

// #[test]
// fn regex_vs_manual() {
//     let test_string = "
//     % % somuchtolove prelude match x
//     {
//         prelude somuchtolove % % for j in [0 ; 255]
//         { % % j % % => { repeat_ % % j % % () ; } } % % endtimesarecoming postlude _ =>
//         { println ! (\"No match was found!!\") ; }
//     } postlude endtimesarecoming % %";
    
//     let pre_regex = Regex::new(r"^\s*?%\s*?%\s*?(?P<prelude_ident>\w*?)\s*?prelude\s").unwrap();
//     let post_regex = Regex::new(r"\spostlude\s*?(?P<postlude_ident>\w*?)\s*?%\s*?%\s*?$").unwrap();

//     let start_regex = Instant::now();

//     let pre_captures = pre_regex.captures(test_string);
//     let post_captures = post_regex.captures(test_string);

//     let prelude = &pre_captures.unwrap()["prelude_ident"];
//     let postlude = &post_captures.unwrap()["postlude_ident"];
//     let end_regex = Instant::now();

//     println!("{}", prelude);
//     println!("{}", postlude);
//     println!("Regex Took: {:?}", end_regex-start_regex);

    
//     let start_manual = Instant::now();
//     let prelude_identifier = parse_identifier(test_string.chars(), false);
//     let prelude = if let Some(prelude_identifier) = &prelude_identifier {
//         parse_prelude(test_string, prelude_identifier)
//     }
//     else {
//         None
//     };
    
//     let postlude_identifier = parse_identifier(test_string.chars(), true);
//     let end_manual = Instant::now();
    
//     println!("{:?}", prelude_identifier);
//     println!("{:?}", prelude);
//     println!("{:?}", postlude_identifier);
//     println!("Manual Took: {:?}", end_manual-start_manual);
// }

// fn parse_prelude(test_string: &str, identifier: &str) -> Option<(usize, String, usize)> {
//     let original_identifier = identifier;
//     let mut lead_identifier_iter = identifier.chars();
//     let mut state = PreludeParseState::FirstPercent;
//     let mut prelude = Some((0, 0));
//     for (i, c) in test_string.chars().enumerate() {
//         let new_state = state.parse(c, &mut lead_identifier_iter);
//         match new_state {
//             PreludeParseState::Prelude => {
//                 let wip_prelude = prelude.as_mut().unwrap();
//                 wip_prelude.1 = i;
//                 state = new_state;
//             }
//             PreludeParseState::StartPrelude => {
//                 if c.is_alphabetic() {
//                     let wip_prelude = prelude.as_mut().unwrap();
//                     wip_prelude.0 = i;
//                     wip_prelude.1 = i;
//                 }
//                 state = new_state;
//             }
//             PreludeParseState::Complete => {
//                 break;
//             }
//             PreludeParseState::NoLude => {
//                 prelude = None;
//                 break;
//             }
//             _ => {
//                 state = new_state;
//             }
//         }
//     }
//     if let Some((start, end)) = prelude {
//         Some((start, test_string[start..end+1].to_string(), end))
//     }
//     else {
//         None
//     }
// }

// #[derive(Debug)]
// enum PreludeParseState {
//     FirstPercent,
//     SecondPercent,
//     StartIdentifier,
//     Identifier,
//     StartPreludeKeyword,
//     PreludeKeyword,
//     StartPrelude,
//     Prelude,
//     Complete,
//     NoLude,
// }

// impl PreludeParseState {
//     fn parse(&self, c: char, mut identifier: impl Iterator<Item = char>) -> Self {
//         match self {
//             Self::FirstPercent => {
//                 if c == '%' {
//                     Self::SecondPercent
//                 } else if c.is_whitespace() {
//                     Self::FirstPercent
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::SecondPercent => {
//                 if c == '%' {
//                     Self::StartIdentifier
//                 } else if c.is_whitespace() {
//                     Self::SecondPercent
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::StartIdentifier => {
//                 if c.is_alphabetic() {
//                     let next_char_in_identifier = identifier.next();
//                     if next_char_in_identifier.is_some() && c == next_char_in_identifier.unwrap() {
//                         Self::Identifier
//                     }
//                     else {
//                         Self::NoLude
//                     }
//                 } else if c.is_whitespace() {
//                     Self::StartIdentifier
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::Identifier => {
//                 let next_char_in_identifier = identifier.next();
//                 if c.is_whitespace() && next_char_in_identifier.is_none() {
//                     Self::StartPrelude
//                 }
//                 else if next_char_in_identifier.is_some() && c == next_char_in_identifier.unwrap() {
//                     Self::StartPreludeKeyword
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::StartPreludeKeyword => {
//                 if c.is_alphabetic() {
//                     Self::Prelude
//                 } else if c.is_whitespace() {
//                     Self::StartPrelude
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::PreludeKeyword => {
//                 if c.is_whitespace() {
//                     Self::PreludeKeyword
//                 } else if c.is_alphabetic() {
//                     Self::StartPrelude
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::StartPrelude => {
//                 if c.is_alphabetic() {
//                     Self::Prelude
//                 } else if c.is_whitespace() {
//                     Self::StartPrelude
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::Prelude => {
//                 if c.is_whitespace() {
//                     Self::Complete
//                 } else if c.is_alphabetic() {
//                     Self::Prelude
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             _ => panic!("Already terminated parsing"),
//         }
//     }
// }


// fn parse_identifier(mut test_string: Chars, reverse: bool) -> Option<String> {
//     let mut state = ParseState::FirstPercent;
//     let mut identifier = Some(String::new());
//     let mut rev_iter;
//     let test_string: &mut dyn Iterator<Item = char> = if reverse {
//         rev_iter = test_string.rev();
//         &mut rev_iter
//     }
//     else {
//         &mut test_string
//     };
//     for c in test_string {
//         let new_state = state.parse(c);
//         match new_state {
//             ParseState::Identifier => {
//                 let wip_identifier = identifier.as_mut().unwrap();
//                 if reverse { wip_identifier.insert(0, c); } else { wip_identifier.push(c) };
//                 state = new_state;
//             }
//             ParseState::StartIdentifier => {
//                 if c.is_alphabetic() {
//                     let wip_identifier = identifier.as_mut().unwrap();
//                     if reverse { wip_identifier.insert(0, c); } else { wip_identifier.push(c); };
//                 }
//                 state = new_state;
//             }
//             ParseState::Complete => {
//                 break;
//             }
//             ParseState::NoLude => {
//                 identifier = None;
//                 break;
//             }
//             _ => {
//                 state = new_state;
//             }
//         }
//     }
//     identifier
// }

// #[derive(Debug)]
// enum ParseState {
//     FirstPercent,
//     SecondPercent,
//     StartIdentifier,
//     Identifier,
//     Complete,
//     NoLude,
// }

// impl ParseState {
//     fn parse(&self, c: char) -> Self {
//         match self {
//             Self::FirstPercent => {
//                 if c == '%' {
//                     Self::SecondPercent
//                 } else if c.is_whitespace() {
//                     Self::FirstPercent
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::SecondPercent => {
//                 if c == '%' {
//                     Self::StartIdentifier
//                 } else if c.is_whitespace() {
//                     Self::SecondPercent
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::StartIdentifier => {
//                 if c.is_whitespace() {
//                     Self::StartIdentifier
//                 } else if c.is_alphabetic() {
//                     Self::Identifier
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             Self::Identifier => {
//                 if c.is_alphabetic() {
//                     Self::Identifier
//                 } else if c.is_whitespace() {
//                     Self::Complete
//                 } else {
//                     Self::NoLude
//                 }
//             }
//             _ => panic!("Already terminated parsing"),
//         }
//     }
// }
