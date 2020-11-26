#[macro_use]
extern crate lazy_static;
use std::{collections::HashMap, fs::File, sync::RwLock};
mod ident_parser;
use ident_parser::parse_identifier;
use proc_macro::TokenStream;
use proc_macro2::Span;
use regex::Regex;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, LitStr, Result, Token,
};

struct RepeatDefinition {
    replacement_id: Ident,
    range: RepeatRange,
    repeat_block: proc_macro2::TokenStream,
    separator: LitStr,
}

struct RepeatRange {
    low: LitInt,
    high: LitInt,
    step: Option<LitInt>,
}

impl Parse for RepeatRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let _repeated_flame = flame::start_guard("RepeatRange::parse");
        let range;
        let _ = bracketed!(range in input);
        let mut range: Vec<_> =
            syn::punctuated::Punctuated::<syn::LitInt, Token![;]>::parse_terminated(&range)?
                .into_iter()
                .collect();
        range.reverse();
        Ok(RepeatRange {
            low: range.pop().unwrap(),
            high: range.pop().unwrap(),
            step: range.pop(),
        })
    }
}

impl Parse for RepeatDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let _repeated_flame = flame::start_guard("RepeatDefinition::parse");
        // Parse repeated for
        let _: Token![for] = input.parse()?;
        let replacement_id: Ident = input.parse()?;
        let _: Token![in] = input.parse()?;
        let range: RepeatRange = input.parse()?;
        let repeat_block;
        let _ = braced!(repeat_block in input);
        let repeat_block = repeat_block.parse()?;

        // Parse Separator if provided
        let mut separator = LitStr::new("\n", Span::call_site());
        let expecting_user_separator = input.parse::<Token![,]>();
        if let Ok(_) = expecting_user_separator {
            separator = input.parse()?
        }
        Ok(RepeatDefinition {
            replacement_id,
            range,
            repeat_block,
            separator,
        })
    }
}

lazy_static! {
    static ref PRELUDE_IDENT_RE: Regex =
        Regex::new(r"^\s*?%\s*?%\s*?(?P<prelude_ident>\w*?)\s*?prelude\s").unwrap();
    static ref POSTLUDE_IDENT_RE: Regex =
        Regex::new(r"\spostlude\s*?(?P<postlude_ident>\w*?)\s*?%\s*?%\s*?$").unwrap();
    static ref SIMPLE_BODY_RE: Regex = Regex::new(r"(?s)(?P<body>.*)").unwrap();
    static ref PRELUDE_REGEX_CACHE: RwLock<HashMap<String, Regex>> = RwLock::new(HashMap::new());
    static ref POSTLUDE_REGEX_CACHE: RwLock<HashMap<String, Regex>> = RwLock::new(HashMap::new());
    static ref BODY_BOTH_REGEX_CACHE: RwLock<HashMap<(String, String), Regex>> =
        RwLock::new(HashMap::new());
    static ref BODY_PRELUDE_REGEX_CACHE: RwLock<HashMap<String, Regex>> =
        RwLock::new(HashMap::new());
    static ref BODY_POSTLUDE_REGEX_CACHE: RwLock<HashMap<String, Regex>> =
        RwLock::new(HashMap::new());
    static ref REPITITION_ID_REGEX_CACHE: RwLock<HashMap<String, Regex>> =
        RwLock::new(HashMap::new());
}

#[proc_macro]
pub fn repeated(repeated_tokens: TokenStream) -> TokenStream {
    flame::start("repeated");
    let tokens_string = format!("{}", repeated_tokens);
    for rt in repeated_tokens {
        println!("{:?}", rt);
    }
    println!("{}", tokens_string);

    // let mut prelude_ident = None;
    let mut prelude_string = String::new();
    let prelude_ident = parse_identifier(tokens_string.chars(), false);
    if let Some(prelude_ident) = &prelude_ident {
        // prelude_ident = Some(prelude_ident_captures["prelude_ident"].to_string());
        let regex_cache = PRELUDE_REGEX_CACHE.read().unwrap();
        let prelude_re = regex_cache.get(prelude_ident);
        let prelude_body_captures = if let None = prelude_re {
            drop(regex_cache);
            PRELUDE_REGEX_CACHE
                .write()
                .unwrap()
                .entry(prelude_ident.to_string())
                .or_insert_with(|| {
                    Regex::new(&format!(
                r"(?s)^\s*?%\s*?%\s*?{}\s*?prelude\s?(?P<prelude>.*)\s?prelude\s*?{}\s*?%\s*?%",
                prelude_ident,
                prelude_ident
            ))
            .unwrap()
                })
                .captures(&tokens_string)
        } else {
            prelude_re.unwrap().captures(&tokens_string)
        };
        let prelude_body = &prelude_body_captures.unwrap()["prelude"];
        prelude_string = prelude_body.into();
    }

    // let mut postlude_ident = None;
    let mut postlude_string = String::new();
    let postlude_ident = parse_identifier(tokens_string.chars(), true);
    if let Some(postlude_ident) = &postlude_ident {
        // postlude_ident = Some(postlude_ident_captures["postlude_ident"].to_string());
        let regex_cache = POSTLUDE_REGEX_CACHE.read().unwrap();
        let postlude_re = regex_cache.get(postlude_ident);
        let postlude_body_captures = if let None = postlude_re {
            drop(regex_cache);
            POSTLUDE_REGEX_CACHE
                .write()
                .unwrap()
                .entry(postlude_ident.to_string())
                .or_insert_with(|| {
                    Regex::new(&format!(
                        r"(?s)%\s*?%\s*?{}\s*?postlude\s?(?P<postlude>.*)\s?postlude\s*?{}\s*?%\s*?%\s*?$",
                        postlude_ident,
                        postlude_ident
        )).unwrap()
                }).captures(&tokens_string)
        } else {
            postlude_re.unwrap().captures(&tokens_string)
        };
        let postlude_body = &postlude_body_captures.unwrap()["postlude"];
        postlude_string = postlude_body.into();
    }

    // Figure out what the repeated body template is
    let mut rep_body_string = None;
    match (prelude_ident, postlude_ident) {
        (None, None) => {}
        (Some(prelude_ident), Some(postlude_ident)) => {
            let regex_cache = BODY_BOTH_REGEX_CACHE.read().unwrap();
            let body_re = regex_cache.get(&(prelude_ident.clone(), postlude_ident.clone()));
            let body_captures = if let None = body_re {
                drop(regex_cache);
                BODY_BOTH_REGEX_CACHE
                .write()
                .unwrap()
                .entry((prelude_ident.clone(), postlude_ident.clone()))
                .or_insert_with(|| {
                    Regex::new(&format!(r"(?xs)
                        ^\s*?%\s*?%\s*?{}\s*?prelude\s?(?P<prelude>.*)\s?prelude\s*?{}\s*?%\s*?%       # Prelude
                        (?P<body>.*?)                                                                # Repitition Body
                        %\s*?%\s*?{}\s*?postlude\s?(?P<postlude>.*)\s?postlude\s*?{}\s*?%\s*?%\s*?$    # Postlude",
                        prelude_ident, prelude_ident, postlude_ident, postlude_ident)).unwrap()
                }).captures(&tokens_string).unwrap()
            } else {
                body_re.unwrap().captures(&tokens_string).unwrap()
            };
            rep_body_string = Some((&body_captures["body"]).to_string());
        }
        (Some(prelude_ident), None) => {
            let regex_cache = BODY_PRELUDE_REGEX_CACHE.read().unwrap();
            let body_re = regex_cache.get(&prelude_ident);
            let body_captures = if let None = body_re {
                drop(regex_cache);
                BODY_PRELUDE_REGEX_CACHE
                .write()
                .unwrap()
                .entry(prelude_ident.clone())
                .or_insert_with(|| {
                    Regex::new(&format!(r"(?xs)
            ^\s*?%\s*?%\s*?{}\s*?prelude\s?(?P<prelude>.*)\s?prelude\s*?{}\s*?%\s*?%       # Prelude
            (?P<body>.*?)                                                                # Repitition Body",
            prelude_ident, prelude_ident)).unwrap()
        }).captures(&tokens_string).unwrap()
            } else {
                body_re.unwrap().captures(&tokens_string).unwrap()
            };
            rep_body_string = Some((&body_captures["body"]).to_string());
        }
        (None, Some(postlude_ident)) => {
            let regex_cache = BODY_POSTLUDE_REGEX_CACHE.read().unwrap();
            let body_re = regex_cache.get(&postlude_ident);
            let body_captures = if let None = body_re {
                drop(regex_cache);
                BODY_POSTLUDE_REGEX_CACHE
                .write()
                .unwrap()
                .entry(postlude_ident.clone())
                .or_insert_with(|| {
                    Regex::new(&format!(r"(?xs)
            (?P<body>.*?)                                                                # Repitition Body
            %\s*?%\s*?{}\s*?postlude\s?(?P<postlude>.*)\s?postlude\s*?{}\s*?%\s*?%\s*?$    # Postlude",
            postlude_ident, postlude_ident)).unwrap()
        }).captures(&tokens_string).unwrap()
            } else {
                body_re.unwrap().captures(&tokens_string).unwrap()
            };
            rep_body_string = Some((&body_captures["body"]).to_string());
        }
    }

    let rep_body_tokenstream = if let Some(rep_body_string) = rep_body_string {
        let _repeated_flame = flame::start_guard("tokenstream body");
        // TODO: Get the generated tokenstreams to use the spans from the original tokenstream so that we can indicate syntax errors more easily!
        TokenStream::from(
            rep_body_string
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|e| {
                    let e: syn::Error = e.into();
                    e.to_compile_error()
                }),
        )
    } else {
        let _repeated_flame = flame::start_guard("simple");
        // repeated_tokens
        TokenStream::new()
    };
    let repitition_definition: RepeatDefinition = parse_macro_input!(rep_body_tokenstream);
    let r = generate_repititions(prelude_string, repitition_definition, postlude_string)
        .unwrap_or_else(|e| TokenStream::from(e.to_compile_error()));

    flame::end("repeated");
    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
    r
}

fn generate_repititions(
    prelude_string: String,
    repitition_definition: RepeatDefinition,
    postlude_string: String,
) -> Result<TokenStream> {
    let _repeated_flame = flame::start_guard("generate_repititions");
    // Determine our range and step_by parameters for repititions
    let start = repitition_definition.range.low.base10_parse::<u32>()?;
    let end = repitition_definition.range.high.base10_parse::<u32>()?;
    let step = repitition_definition
        .range
        .step
        .map_or(Ok(1), |s| s.base10_parse::<usize>())?;

    // Generate string versions of repeated tokenstream
    let replacement_id = repitition_definition.replacement_id.to_string();
    let replacement_id_regex_replacer_cache = REPITITION_ID_REGEX_CACHE.read().unwrap();
    let replacement_id_regex_replacer = replacement_id_regex_replacer_cache.get(&replacement_id);
    let mut replacement_id_write_cache;
    let replacement_id_regex_replacer = if let None = replacement_id_regex_replacer {
        drop(replacement_id_regex_replacer_cache);
        replacement_id_write_cache = REPITITION_ID_REGEX_CACHE.write().unwrap();
        replacement_id_write_cache
            .entry(replacement_id.clone())
            .or_insert_with(|| {
                Regex::new(&format!(r"\s*?%\s*?%\s*?{}\s*?%\s*?%\s*?", replacement_id)).unwrap()
            })
    } else {
        replacement_id_regex_replacer.unwrap()
    };
    let repeated_block: String = repitition_definition.repeat_block.to_string();
    let mut repeated_blocks = if prelude_string.is_empty() {
        Vec::new()
    } else {
        vec![prelude_string]
    };
    for i in (start..=end).step_by(step) {
        let new_block =
            replacement_id_regex_replacer.replace_all(&repeated_block, i.to_string().as_str());
        repeated_blocks.push(new_block.into());
    }
    if !postlude_string.is_empty() {
        repeated_blocks.push(postlude_string);
    }

    // Build final tokenstream by joining all repeated tokenstreams with the prelude and the postlude
    let separator = repitition_definition.separator.value();
    flame::start("parse all");
    let repeated_tokenstream: proc_macro2::TokenStream =
        repeated_blocks.join(&separator).parse()?;
    flame::end("parse all");
    Ok(TokenStream::from(repeated_tokenstream))
}
