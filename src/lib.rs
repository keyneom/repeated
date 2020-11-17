#[macro_use]
extern crate lazy_static;
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
}

#[proc_macro]
pub fn repeated(repeated_tokens: TokenStream) -> TokenStream {
    let tokens_string = format!("{}", repeated_tokens);

    let mut prelude_ident = None;
    let mut prelude_string = String::new();
    let prelude_ident_captures = PRELUDE_IDENT_RE.captures(&tokens_string);
    if let Some(prelude_ident_captures) = prelude_ident_captures {
        println!("{:?}", prelude_ident_captures);
        prelude_ident = Some(prelude_ident_captures["prelude_ident"].to_string());
        let prelude_re = Regex::new(&format!(
            r"(?s)^\s*?%\s*?%\s*?{}\s*?prelude\s(?P<prelude>.*)\sprelude\s*?{}\s*?%\s*?%",
            prelude_ident.as_ref().unwrap(),
            prelude_ident.as_ref().unwrap()
        ))
        .unwrap();
        let prelude_body_captures = prelude_re.captures(&tokens_string);
        let prelude_body = &prelude_body_captures.unwrap()["prelude"];
        prelude_string = prelude_body.into();
        println!("PRELUDE_RE: {}", prelude_body);
    }

    let mut postlude_ident = None;
    let mut postlude_string = String::new();
    let postlude_ident_captures = POSTLUDE_IDENT_RE.captures(&tokens_string);
    if let Some(postlude_ident_captures) = postlude_ident_captures {
        println!("{:?}", postlude_ident_captures);
        postlude_ident = Some(postlude_ident_captures["postlude_ident"].to_string());
        let postlude_re = Regex::new(&format!(
            r"(?s)%\s*?%\s*?{}\s*?postlude\s(?P<postlude>.*)\spostlude\s*?{}\s*?%\s*?%\s*?$",
            postlude_ident.as_ref().unwrap(),
            postlude_ident.as_ref().unwrap()
        ))
        .unwrap();
        let postlude_body_captures = postlude_re.captures(&tokens_string);
        let postlude_body = &postlude_body_captures.unwrap()["postlude"];
        postlude_string = postlude_body.into();
        println!("POSTLUDE_RE: {}", postlude_body);
    }

    // Figure out what the repeated body tempalte is
    let mut rep_body_string = None;
    match (prelude_ident, postlude_ident) {
        (None, None) => {
            
        }
        (Some(prelude_ident), Some(postlude_ident)) => {
            let body_re = Regex::new(&format!(r"(?xs)
            ^\s*?%\s*?%\s*?{}\s*?prelude\s(?P<prelude>.*)\sprelude\s*?{}\s*?%\s*?%       # Prelude
            (?P<body>.*?)                                                                # Repitition Body
            %\s*?%\s*?{}\s*?postlude\s(?P<postlude>.*)\spostlude\s*?{}\s*?%\s*?%\s*?$    # Postlude",
            prelude_ident, prelude_ident, postlude_ident, postlude_ident)).unwrap();
            let body_captures = body_re.captures(&tokens_string).unwrap();
            rep_body_string = Some((&body_captures["body"]).to_string());
            println!("BODY_RE: {:?}", rep_body_string);
        }
        (Some(prelude_ident), None) => {
            let body_re = Regex::new(&format!(r"(?xs)
            ^\s*?%\s*?%\s*?{}\s*?prelude\s(?P<prelude>.*)\sprelude\s*?{}\s*?%\s*?%       # Prelude
            (?P<body>.*?)                                                                # Repitition Body",
            prelude_ident, prelude_ident)).unwrap();
            let body_captures = body_re.captures(&tokens_string).unwrap();
            rep_body_string = Some((&body_captures["body"]).to_string());
            println!("BODY_RE: {:?}", rep_body_string);
        }
        (None, Some(postlude_ident)) => {
            let body_re = Regex::new(&format!(r"(?xs)
            (?P<body>.*?)                                                                # Repitition Body
            %\s*?%\s*?{}\s*?postlude\s(?P<postlude>.*)\spostlude\s*?{}\s*?%\s*?%\s*?$    # Postlude",
            postlude_ident, postlude_ident)).unwrap();
            let body_captures = body_re.captures(&tokens_string).unwrap();
            rep_body_string = Some((&body_captures["body"]).to_string());
            println!("BODY_RE: {:?}", rep_body_string);
        }
    }

    // TODO: Get the generated tokenstreams to use the spans from the original tokenstream so that we can indicate syntax errors more easily!
    let rep_body_tokenstream = if let Some(rep_body_string) = rep_body_string { TokenStream::from(
        rep_body_string
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|e| {
                let e: syn::Error = e.into();
                e.to_compile_error()
            }),
    ) } else {
        repeated_tokens
    };
    let repitition_definition: RepeatDefinition = parse_macro_input!(rep_body_tokenstream);
    generate_repititions(prelude_string, repitition_definition, postlude_string)
        .unwrap_or_else(|e| TokenStream::from(e.to_compile_error()))
}

fn generate_repititions(
    prelude_string: String,
    repitition_definition: RepeatDefinition,
    postlude_string: String,
) -> Result<TokenStream> {
    // Determine our range and step_by parameters for repititions
    let start = repitition_definition.range.low.base10_parse::<u32>()?;
    let end = repitition_definition.range.high.base10_parse::<u32>()?;
    let step = repitition_definition
        .range
        .step
        .map_or(Ok(1), |s| s.base10_parse::<usize>())?;

    // Generate string versions of repeated tokenstream
    let replacement_id = repitition_definition.replacement_id.to_string();
    let repeated_block: String = repitition_definition.repeat_block.to_string();
    let mut repeated_blocks = vec![prelude_string];
    for i in (start..=end).step_by(step) {
        let new_block = repeated_block
            .replace(&format!(" % % {} % % ", replacement_id), &format!("{}", i))
            .replace(&format!("% % {} % %", replacement_id), &format!("{}", i))
            .replace(&format!("%%{}%%", replacement_id), &format!("{}", i));
        repeated_blocks.push(new_block);
    }
    repeated_blocks.push(postlude_string);

    // Build final tokenstream by joining all repeated tokenstreams with the prelude and the postlude
    let separator = repitition_definition.separator.value();
    let repeated_tokenstream: proc_macro2::TokenStream =
        repeated_blocks.join(&separator).parse()?;
    Ok(TokenStream::from(repeated_tokenstream))
}
