use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Ident, Literal, Spacing, Span, TokenTree};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, LitInt, LitStr, Result, Token,
};

struct RepeatedInput {
    prelude: Vec<FlatToken>,
    definition: RepeatDefinition,
    postlude: Vec<FlatToken>,
}

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
        let content;
        let bracket = bracketed!(content in input);
        let values: Vec<LitInt> =
            syn::punctuated::Punctuated::<LitInt, Token![;]>::parse_terminated(&content)?
                .into_iter()
                .collect();

        match values.len() {
            2 => Ok(RepeatRange {
                low: values[0].clone(),
                high: values[1].clone(),
                step: None,
            }),
            3 => Ok(RepeatRange {
                low: values[0].clone(),
                high: values[1].clone(),
                step: Some(values[2].clone()),
            }),
            0 | 1 => Err(syn::Error::new(
                bracket.span,
                "repeat range requires [low; high] or [low; high; step]",
            )),
            _ => Err(syn::Error::new(
                bracket.span,
                "repeat range expects at most three values: [low; high; step]",
            )),
        }
    }
}

impl Parse for RepeatDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Token![for] = input.parse()?;
        let replacement_id: Ident = input.parse()?;
        let _: Token![in] = input.parse()?;
        let range: RepeatRange = input.parse()?;
        let block_content;
        braced!(block_content in input);
        let repeat_block = block_content.parse()?;

        let separator = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            input.parse()?
        } else {
            LitStr::new("\n", Span::call_site())
        };

        Ok(RepeatDefinition {
            replacement_id,
            range,
            repeat_block,
            separator,
        })
    }
}

impl Parse for RepeatedInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let stream = input.parse::<proc_macro2::TokenStream>()?;
        split_prelude_postlude(stream)
    }
}

/// Flat view of a token stream so prelude/postlude markers can sit across
/// unbalanced delimiter pairs (e.g. `match x {` in the prelude, `}` in the postlude).
#[derive(Clone, Debug)]
enum FlatToken {
    Token(TokenTree),
    Start(Delimiter, Span),
    End(Delimiter, Span),
}

fn flatten(stream: proc_macro2::TokenStream) -> Vec<FlatToken> {
    let mut out = Vec::new();
    for tt in stream {
        match tt {
            TokenTree::Group(group) => {
                out.push(FlatToken::Start(group.delimiter(), group.span_open()));
                out.extend(flatten(group.stream()));
                out.push(FlatToken::End(group.delimiter(), group.span_close()));
            }
            other => out.push(FlatToken::Token(other)),
        }
    }
    out
}

fn unflatten(flat: &[FlatToken]) -> Result<proc_macro2::TokenStream> {
    fn rec(flat: &[FlatToken], index: &mut usize) -> Result<proc_macro2::TokenStream> {
        let mut out = proc_macro2::TokenStream::new();
        while *index < flat.len() {
            match &flat[*index] {
                FlatToken::End(_, _) => return Ok(out),
                FlatToken::Start(delimiter, open_span) => {
                    let delimiter = *delimiter;
                    let open_span = *open_span;
                    *index += 1;
                    let inner = rec(flat, index)?;
                    let close_span = match flat.get(*index) {
                        Some(FlatToken::End(end_delim, span)) if *end_delim == delimiter => {
                            let span = *span;
                            *index += 1;
                            span
                        }
                        Some(FlatToken::End(_, span)) => {
                            return Err(syn::Error::new(
                                *span,
                                "mismatched closing delimiter while reconstructing tokens",
                            ));
                        }
                        _ => {
                            return Err(syn::Error::new(
                                open_span,
                                "unclosed delimiter while reconstructing tokens",
                            ));
                        }
                    };
                    let mut group = Group::new(delimiter, inner);
                    group.set_span(open_span.join(close_span).unwrap_or(open_span));
                    out.extend(std::iter::once(TokenTree::Group(group)));
                }
                FlatToken::Token(tt) => {
                    out.extend(std::iter::once(tt.clone()));
                    *index += 1;
                }
            }
        }
        Ok(out)
    }

    let mut index = 0;
    let stream = rec(flat, &mut index)?;
    if index != flat.len() {
        return Err(syn::Error::new(
            Span::call_site(),
            "internal error: leftover tokens while reconstructing token stream",
        ));
    }
    Ok(stream)
}

fn is_percent(flat: &FlatToken) -> bool {
    matches!(flat, FlatToken::Token(TokenTree::Punct(p)) if p.as_char() == '%')
}

fn as_ident(flat: &FlatToken) -> Option<&Ident> {
    match flat {
        FlatToken::Token(TokenTree::Ident(ident)) => Some(ident),
        _ => None,
    }
}

fn match_percent_percent(flat: &[FlatToken], index: usize) -> bool {
    index + 1 < flat.len() && is_percent(&flat[index]) && is_percent(&flat[index + 1])
}

fn find_closing_marker(
    flat: &[FlatToken],
    start: usize,
    first_label: &str,
    ident: &Ident,
) -> Option<usize> {
    let mut i = start;
    while i + 3 < flat.len() {
        if let (Some(a), Some(b)) = (as_ident(&flat[i]), as_ident(&flat[i + 1])) {
            if a == first_label && b == ident && match_percent_percent(flat, i + 2) {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn split_prelude_postlude(stream: proc_macro2::TokenStream) -> Result<RepeatedInput> {
    let flat = flatten(stream);

    let mut body_start = 0;
    let mut prelude = Vec::new();

    if match_percent_percent(&flat, 0) {
        let prelude_ident = as_ident(flat.get(2).ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "expected identifier after `%%` in prelude marker",
            )
        })?)
        .ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "expected identifier after `%%` in prelude marker",
            )
        })?;
        let prelude_kw = as_ident(flat.get(3).ok_or_else(|| {
            syn::Error::new(
                prelude_ident.span(),
                "expected `prelude` after identifier in prelude marker",
            )
        })?)
        .ok_or_else(|| {
            syn::Error::new(
                prelude_ident.span(),
                "expected `prelude` after identifier in prelude marker",
            )
        })?;
        if prelude_kw != "prelude" {
            return Err(syn::Error::new(
                prelude_kw.span(),
                "expected `prelude` after identifier in prelude marker",
            ));
        }

        let close = find_closing_marker(&flat, 4, "prelude", prelude_ident).ok_or_else(|| {
            syn::Error::new(
                prelude_ident.span(),
                format!(
                    "could not find closing prelude marker `prelude {} %%`",
                    prelude_ident
                ),
            )
        })?;

        prelude = flat[4..close].to_vec();
        body_start = close + 4;
    }

    let mut postlude = Vec::new();
    let mut post_marker = None;
    {
        let mut i = body_start;
        while i + 3 < flat.len() {
            if match_percent_percent(&flat, i) {
                if let (Some(ident), Some(kw)) = (as_ident(&flat[i + 2]), as_ident(&flat[i + 3])) {
                    if kw == "postlude" {
                        post_marker = Some((i, ident.clone()));
                        break;
                    }
                }
            }
            i += 1;
        }
    }

    let body_end = if let Some((post_start, post_ident)) = post_marker {
        let close = find_closing_marker(&flat, post_start + 4, "postlude", &post_ident)
            .ok_or_else(|| {
                syn::Error::new(
                    post_ident.span(),
                    format!(
                        "could not find closing postlude marker `postlude {} %%`",
                        post_ident
                    ),
                )
            })?;
        if close + 4 != flat.len() {
            return Err(syn::Error::new(
                post_ident.span(),
                "tokens found after closing postlude marker",
            ));
        }
        postlude = flat[post_start + 4..close].to_vec();
        post_start
    } else {
        flat.len()
    };

    // The for-body slice is balanced on its own, so it can be re-grouped safely.
    let body_stream = unflatten(&flat[body_start..body_end])?;
    let definition: RepeatDefinition = syn::parse2(body_stream)?;

    Ok(RepeatedInput {
        prelude,
        definition,
        postlude,
    })
}

fn is_percent_tt(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == '%')
}

/// If `tokens[index..]` starts with `%% <id> %%`, returns the index past the marker.
fn match_placeholder_at(tokens: &[TokenTree], index: usize, replacement_id: &str) -> Option<usize> {
    if index + 4 >= tokens.len() {
        return None;
    }
    if !is_percent_tt(&tokens[index]) || !is_percent_tt(&tokens[index + 1]) {
        return None;
    }
    match &tokens[index + 2] {
        TokenTree::Ident(ident) if ident == replacement_id => {}
        _ => return None,
    }
    if !is_percent_tt(&tokens[index + 3]) || !is_percent_tt(&tokens[index + 4]) {
        return None;
    }
    Some(index + 5)
}

fn substitute_literal(lit: &Literal, replacement_id: &str, value: u32) -> TokenTree {
    let rendered = lit.to_string();
    let needle = format!("%%{}%%", replacement_id);
    if !rendered.contains(&needle) {
        return TokenTree::Literal(lit.clone());
    }
    let replaced = rendered.replace(&needle, &value.to_string());
    match replaced.parse::<proc_macro2::TokenStream>() {
        Ok(stream) => stream
            .into_iter()
            .next()
            .unwrap_or_else(|| TokenTree::Literal(lit.clone())),
        Err(_) => TokenTree::Literal(lit.clone()),
    }
}

fn substitute_stream(
    input: proc_macro2::TokenStream,
    replacement_id: &str,
    value: u32,
) -> proc_macro2::TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut out = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if let TokenTree::Ident(ident) = &tokens[i] {
            if let Some(end) = match_placeholder_at(&tokens, i + 1, replacement_id) {
                let name = format!("{}{}", ident, value);
                out.push(TokenTree::Ident(Ident::new(&name, ident.span())));
                i = end;
                continue;
            }
        }

        if let Some(end) = match_placeholder_at(&tokens, i, replacement_id) {
            let mut lit = Literal::u32_unsuffixed(value);
            if let TokenTree::Punct(percent) = &tokens[i] {
                lit.set_span(percent.span());
            }
            out.push(TokenTree::Literal(lit));
            i = end;
            continue;
        }

        match &tokens[i] {
            TokenTree::Group(group) => {
                let inner = substitute_stream(group.stream(), replacement_id, value);
                let mut new_group = Group::new(group.delimiter(), inner);
                new_group.set_span(group.span());
                out.push(TokenTree::Group(new_group));
            }
            TokenTree::Literal(lit) => {
                out.push(substitute_literal(lit, replacement_id, value));
            }
            other => out.push(other.clone()),
        }
        i += 1;
    }

    out.into_iter().collect()
}

fn flat_to_string(flat: &[FlatToken]) -> String {
    let mut s = String::new();
    for token in flat {
        match token {
            FlatToken::Token(TokenTree::Punct(punct)) => {
                s.push(punct.as_char());
                if punct.spacing() == Spacing::Alone {
                    s.push(' ');
                }
            }
            FlatToken::Token(other) => {
                s.push_str(&other.to_string());
                s.push(' ');
            }
            FlatToken::Start(Delimiter::Parenthesis, _) => s.push('('),
            FlatToken::Start(Delimiter::Brace, _) => s.push('{'),
            FlatToken::Start(Delimiter::Bracket, _) => s.push('['),
            FlatToken::Start(Delimiter::None, _) => {}
            FlatToken::End(Delimiter::Parenthesis, _) => {
                s.push(')');
                s.push(' ');
            }
            FlatToken::End(Delimiter::Brace, _) => {
                s.push('}');
                s.push(' ');
            }
            FlatToken::End(Delimiter::Bracket, _) => {
                s.push(']');
                s.push(' ');
            }
            FlatToken::End(Delimiter::None, _) => {}
        }
    }
    s
}

impl RepeatedInput {
    fn expand(self) -> Result<proc_macro2::TokenStream> {
        let RepeatedInput {
            prelude,
            definition,
            postlude,
        } = self;

        let start = definition.range.low.base10_parse::<u32>()?;
        let end = definition.range.high.base10_parse::<u32>()?;
        let step = match &definition.range.step {
            Some(step) => step.base10_parse::<usize>()?,
            None => 1,
        };
        if step == 0 {
            return Err(syn::Error::new(
                definition
                    .range
                    .step
                    .as_ref()
                    .map(|s| s.span())
                    .unwrap_or_else(Span::call_site),
                "repeat step must be non-zero",
            ));
        }

        let replacement_id = definition.replacement_id.to_string();
        let separator = definition.separator.value();

        // Prelude/postlude stay flat so braces opened in the prelude and closed
        // in the postlude are only re-parsed after joining. Bodies are substituted
        // via a token-tree walk — preserving empty-separator gluing
        // (e.g. `{1}` joined with `""` → `1111`).
        let mut parts: Vec<String> = Vec::new();
        if !prelude.is_empty() {
            parts.push(flat_to_string(&prelude));
        }
        for value in (start..=end).step_by(step) {
            let substituted = substitute_stream(
                definition.repeat_block.clone(),
                &replacement_id,
                value,
            );
            parts.push(substituted.to_string());
        }
        if !postlude.is_empty() {
            parts.push(flat_to_string(&postlude));
        }

        parts
            .join(&separator)
            .parse()
            .map_err(|err: proc_macro2::LexError| {
                syn::Error::new(
                    Span::call_site(),
                    format!("failed to parse expanded tokens: {}", err),
                )
            })
    }
}

#[proc_macro]
pub fn repeated(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RepeatedInput);
    input
        .expand()
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
