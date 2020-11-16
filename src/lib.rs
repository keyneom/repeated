use proc_macro::TokenStream;
use proc_macro2::Span;
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

fn split_once<'a>(in_string: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    let mut splitter = in_string.splitn(2, pat);
    let first = splitter.next()?;
    let second = splitter.next()?;
    Some((first, second))
}

#[proc_macro]
pub fn repeated(repeated_tokens: TokenStream) -> TokenStream {
    let tokens_string = format!("{}", repeated_tokens);
    let mut the_rest = None;
    let mut prelude_string = "".to_string();
    let s = split_once(&tokens_string, "% % prelude");
    if let Some(prelude_check) = s {
        if let Some((prelude_body, rest)) = split_once(prelude_check.1, "prelude % %") {
            prelude_string = prelude_body.to_string();
            the_rest = Some(rest);
        }
    }

    let mut postlude_string = "".to_string();
    let s = split_once(
        if let Some(st) = the_rest {
            st
        } else {
            &tokens_string
        },
        "% % postlude",
    );
    if let Some(postlude_check) = s {
        the_rest = Some(postlude_check.0);
        if let Some((postlude_body, _)) = split_once(postlude_check.1, "postlude % %") {
            postlude_string = postlude_body.to_string();
        }
    }

    if let Some(rep_body) = the_rest {
        let rep_body_tokenstream = TokenStream::from(
            rep_body
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|e| {
                    let e: syn::Error = e.into();
                    e.to_compile_error()
                }),
        );
        let repitition_definition: RepeatDefinition = parse_macro_input!(rep_body_tokenstream);
        generate_repititions(prelude_string, repitition_definition, postlude_string)
            .unwrap_or_else(|e| TokenStream::from(e.to_compile_error()))
    } else {
        let repitition_definition: RepeatDefinition = parse_macro_input!(repeated_tokens);
        generate_repititions(prelude_string, repitition_definition, postlude_string)
            .unwrap_or_else(|e| TokenStream::from(e.to_compile_error()))
    }
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
