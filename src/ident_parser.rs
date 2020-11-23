use std::str::Chars;

pub(crate) fn parse_identifier(mut test_string: Chars, reverse: bool) -> Option<String> {
    let mut state = ParseState::FirstPercent;
    let mut identifier = Some(String::new());
    let mut rev_iter;
    let test_string: &mut dyn Iterator<Item = char> = if reverse {
        rev_iter = test_string.rev();
        &mut rev_iter
    }
    else {
        &mut test_string
    };
    for c in test_string {
        let new_state = state.parse(c);
        match new_state {
            ParseState::Identifier => {
                let wip_identifier = identifier.as_mut().unwrap();
                if reverse { wip_identifier.insert(0, c); } else { wip_identifier.push(c) };
                state = new_state;
            }
            ParseState::StartIdentifier => {
                if c.is_alphabetic() {
                    let wip_identifier = identifier.as_mut().unwrap();
                    if reverse { wip_identifier.insert(0, c); } else { wip_identifier.push(c); };
                }
                state = new_state;
            }
            ParseState::Complete => {
                break;
            }
            ParseState::NoLude => {
                identifier = None;
                break;
            }
            _ => {
                state = new_state;
            }
        }
    }
    identifier
}

#[derive(Debug)]
enum ParseState {
    FirstPercent,
    SecondPercent,
    StartIdentifier,
    Identifier,
    Complete,
    NoLude,
}

impl ParseState {
    fn parse(&self, c: char) -> Self {
        match self {
            Self::FirstPercent => {
                if c == '%' {
                    Self::SecondPercent
                } else if c.is_whitespace() {
                    Self::FirstPercent
                } else {
                    Self::NoLude
                }
            }
            Self::SecondPercent => {
                if c == '%' {
                    Self::StartIdentifier
                } else if c.is_whitespace() {
                    Self::SecondPercent
                } else {
                    Self::NoLude
                }
            }
            Self::StartIdentifier => {
                if c.is_whitespace() {
                    Self::StartIdentifier
                } else if c.is_alphabetic() {
                    Self::Identifier
                } else {
                    Self::NoLude
                }
            }
            Self::Identifier => {
                if c.is_alphabetic() {
                    Self::Identifier
                } else if c.is_whitespace() {
                    Self::Complete
                } else {
                    Self::NoLude
                }
            }
            _ => panic!("Already terminated parsing"),
        }
    }
}
