extern crate proc_macro;
use proc_macro::{TokenStream};
use syn::{Ident, LitInt, Result, Token, braced, bracketed, parse::{Parse, ParseStream}, parse_macro_input};

struct RepeatDefinition {
    replacement_id: Ident,
    range: RepeatRange,
    repeat_block: proc_macro2::TokenStream
}

struct RepeatRange {
    low: LitInt,
    high: LitInt,
    step: Option<LitInt>
}

impl Parse for RepeatRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let range;
        let _ = bracketed!(range in input);
        let mut range: Vec<_> = syn::punctuated::Punctuated::<syn::LitInt, Token![;]>::parse_terminated(&range)?.into_iter().collect();
        range.reverse();
        Ok(RepeatRange {
            low: range.pop().unwrap(),
            high: range.pop().unwrap(),
            step: range.pop().and_then(|s| Some(s)),
        })
    }
}

impl Parse for RepeatDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Token![for] = input.parse()?;
        let i: Ident = input.parse()?;
        let _: Token![in] = input.parse()?;
        let range: RepeatRange = input.parse()?;
        let repeat_block;
        let _ = braced!(repeat_block in input);
        let repeat_block = repeat_block.parse()?;
        Ok(RepeatDefinition {
            replacement_id: i,
            range,
            repeat_block,
        })
    }
}

#[proc_macro]
pub fn repeated(repeated_tokens: TokenStream) -> TokenStream {
    println!("item: \"{}\"", repeated_tokens.clone().to_string());
    let repitition_definition = parse_macro_input!(repeated_tokens);
    println!("Parsed definition!");
    generate_repititions(repitition_definition).unwrap_or_else(|e| TokenStream::from(e.to_compile_error()))
}

fn generate_repititions(repitition_definition: RepeatDefinition) -> Result<TokenStream> {
    let start = repitition_definition.range.low.base10_parse::<u32>()?;
    let end = repitition_definition.range.high.base10_parse::<u32>()?;
    let step = repitition_definition.range.step.map_or(Ok(1), |s| s.base10_parse::<usize>())?;

    let replacement_id = repitition_definition.replacement_id.to_string();
    let repeated_block: String = repitition_definition.repeat_block.to_string();
    let mut repeated_blocks = vec![];
    for i in (start..=end).step_by(step) {
        let new_block = repeated_block
        .replace(&format!(" % % {} % % ",replacement_id), &format!("{}", i))
        .replace(&format!("%%{}%%",replacement_id), &format!("{}", i));
        println!("{}", &new_block);
        repeated_blocks.push(new_block);
    }
    let tokenstream_repeated:proc_macro2::TokenStream = repeated_blocks.join("\n").parse()?;
    Ok(TokenStream::from(tokenstream_repeated))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
