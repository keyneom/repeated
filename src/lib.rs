use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Ident, Literal, Spacing, Span, TokenTree};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, Lit, LitInt, LitStr, Result, Token,
};

struct RepeatedInput {
    prelude: Vec<FlatToken>,
    definition: RepeatDefinition,
    postlude: Vec<FlatToken>,
}

struct RepeatDefinition {
    replacement_id: Ident,
    source: RepeatSource,
    repeat_block: proc_macro2::TokenStream,
    separator: LitStr,
}

enum RepeatSource {
    Range {
        low: LitInt,
        high: LitInt,
        step: Option<LitInt>,
    },
    List(Vec<SubstValue>),
}

/// A single value from an enumerated substitution list.
enum SubstValue {
    Ident(Ident),
    Lit(Lit),
}

impl Parse for SubstValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::Ident) {
            Ok(SubstValue::Ident(input.parse()?))
        } else {
            Ok(SubstValue::Lit(input.parse()?))
        }
    }
}

impl SubstValue {
    fn as_replacement(&self) -> Replacement {
        match self {
            SubstValue::Ident(ident) => Replacement {
                text: ident.to_string(),
                token: TokenTree::Ident(ident.clone()),
            },
            SubstValue::Lit(lit) => {
                let token = lit_to_token_tree(lit);
                Replacement {
                    text: token.to_string(),
                    token,
                }
            }
        }
    }
}

fn lit_to_token_tree(lit: &Lit) -> TokenTree {
    match lit {
        Lit::Str(l) => TokenTree::Literal(l.token()),
        Lit::ByteStr(l) => TokenTree::Literal(l.token()),
        Lit::CStr(l) => TokenTree::Literal(l.token()),
        Lit::Byte(l) => TokenTree::Literal(l.token()),
        Lit::Char(l) => TokenTree::Literal(l.token()),
        Lit::Int(l) => TokenTree::Literal(l.token()),
        Lit::Float(l) => TokenTree::Literal(l.token()),
        Lit::Bool(l) => TokenTree::Ident(Ident::new(
            if l.value() { "true" } else { "false" },
            l.span(),
        )),
        Lit::Verbatim(l) => TokenTree::Literal(l.clone()),
        _ => TokenTree::Ident(Ident::new("_", lit.span())),
    }
}

/// Value substituted for `%%id%%` in a repetition.
#[derive(Clone)]
struct Replacement {
    /// Display form used in ident concatenation and string literal rewrite.
    text: String,
    /// Token emitted for a bare `%%id%%` placeholder.
    token: TokenTree,
}

impl Replacement {
    fn from_number(value: u32, span: Span) -> Self {
        let mut lit = Literal::u32_unsuffixed(value);
        lit.set_span(span);
        Self {
            text: value.to_string(),
            token: TokenTree::Literal(lit),
        }
    }

    fn with_span(&self, span: Span) -> Self {
        let mut token = self.token.clone();
        token.set_span(span);
        Self {
            text: self.text.clone(),
            token,
        }
    }
}

impl Parse for RepeatSource {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let bracket = bracketed!(content in input);

        if content.is_empty() {
            return Err(syn::Error::new(
                bracket.span.open(),
                "repeat source cannot be empty; use [low; high], [low; high; step], or [a, b, c]",
            ));
        }

        if top_level_has_semi(&content.fork())? {
            let values: Vec<LitInt> =
                syn::punctuated::Punctuated::<LitInt, Token![;]>::parse_terminated(&content)?
                    .into_iter()
                    .collect();

            return match values.len() {
                2 => Ok(RepeatSource::Range {
                    low: values[0].clone(),
                    high: values[1].clone(),
                    step: None,
                }),
                3 => Ok(RepeatSource::Range {
                    low: values[0].clone(),
                    high: values[1].clone(),
                    step: Some(values[2].clone()),
                }),
                0 | 1 => Err(syn::Error::new(
                    bracket.span.open(),
                    "repeat range requires [low; high] or [low; high; step]",
                )),
                _ => Err(syn::Error::new(
                    bracket.span.open(),
                    "repeat range expects at most three values: [low; high; step]",
                )),
            };
        }

        let values: Vec<SubstValue> =
            syn::punctuated::Punctuated::<SubstValue, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .collect();

        if values.is_empty() {
            return Err(syn::Error::new(
                bracket.span.open(),
                "enumerated substitution list cannot be empty",
            ));
        }

        Ok(RepeatSource::List(values))
    }
}

/// True if the stream contains a top-level `;` (not nested in a group).
fn top_level_has_semi(input: ParseStream) -> Result<bool> {
    while !input.is_empty() {
        if input.peek(Token![;]) {
            return Ok(true);
        }
        let _ = input.parse::<proc_macro2::TokenTree>()?;
    }
    Ok(false)
}

impl Parse for RepeatDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Token![for] = input.parse()?;
        let replacement_id: Ident = input.parse()?;
        let _: Token![in] = input.parse()?;
        let source: RepeatSource = input.parse()?;
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
            source,
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
    flatten_into(stream, &mut out);
    out
}

fn flatten_into(stream: proc_macro2::TokenStream, out: &mut Vec<FlatToken>) {
    for tt in stream {
        match tt {
            TokenTree::Group(group) => {
                out.push(FlatToken::Start(group.delimiter(), group.span_open()));
                flatten_into(group.stream(), out);
                out.push(FlatToken::End(group.delimiter(), group.span_close()));
            }
            other => out.push(FlatToken::Token(other)),
        }
    }
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
        let span = match flat.get(index) {
            Some(FlatToken::Token(tt)) => tt.span(),
            Some(FlatToken::Start(_, span)) | Some(FlatToken::End(_, span)) => *span,
            None => Span::call_site(),
        };
        return Err(syn::Error::new(
            span,
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

fn flat_span_at(flat: &[FlatToken], index: usize) -> Span {
    match flat.get(index) {
        Some(FlatToken::Token(tt)) => tt.span(),
        Some(FlatToken::Start(_, span)) | Some(FlatToken::End(_, span)) => *span,
        None => Span::call_site(),
    }
}

fn split_prelude_postlude(stream: proc_macro2::TokenStream) -> Result<RepeatedInput> {
    let flat = flatten(stream);

    let mut body_start = 0;
    let mut prelude = Vec::new();

    if match_percent_percent(&flat, 0) {
        let open_span = flat_span_at(&flat, 0);
        let prelude_ident = as_ident(flat.get(2).ok_or_else(|| {
            syn::Error::new(
                open_span,
                "expected identifier after `%%` in prelude marker",
            )
        })?)
        .ok_or_else(|| {
            syn::Error::new(
                flat_span_at(&flat, 2.min(flat.len().saturating_sub(1))),
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
                flat_span_at(&flat, close + 4),
                "tokens found after closing postlude marker",
            ));
        }
        postlude = flat[post_start + 4..close].to_vec();
        post_start
    } else {
        flat.len()
    };

    if body_start >= body_end {
        return Err(syn::Error::new(
            flat_span_at(&flat, body_start.min(flat.len().saturating_sub(1))),
            "missing `for ... in ... { ... }` repeat body",
        ));
    }

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

fn match_placeholder_at(
    tokens: &[TokenTree],
    index: usize,
    replacement_id: &str,
) -> Option<(usize, Span)> {
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
    let span = tokens[index]
        .span()
        .join(tokens[index + 4].span())
        .unwrap_or_else(|| tokens[index].span());
    Some((index + 5, span))
}

/// Precompiled repeat-block template: placeholder matching runs once per expansion.
#[derive(Clone)]
enum TemplatePart {
    /// Static token (not a group — groups are [`TemplatePart::Group`]).
    Token(TokenTree),
    /// Bare `%%id%%` → emit replacement token.
    Placeholder { span: Span },
    /// `prefix%%id%%` → emit concatenated ident.
    ConcatIdent { prefix: Ident },
    /// Group whose interior is itself a template.
    Group {
        delimiter: Delimiter,
        span: Span,
        inner: Template,
    },
    /// Literal whose rendered text contains `%%id%%`.
    LitRewrite { lit: Literal, needle: String },
}

#[derive(Clone)]
struct Template {
    parts: Vec<TemplatePart>,
}

impl Template {
    fn compile(input: proc_macro2::TokenStream, replacement_id: &str) -> Self {
        let needle = format!("%%{}%%", replacement_id);
        let tokens: Vec<TokenTree> = input.into_iter().collect();
        let mut parts = Vec::with_capacity(tokens.len());
        let mut i = 0;

        while i < tokens.len() {
            // Concatenate only when the preceding ident ends with `_`
            // (`item_%%n%%` → `item_0`). Plain `for %%T%%` must stay two tokens.
            if let TokenTree::Ident(ident) = &tokens[i] {
                if ident.to_string().ends_with('_') {
                    if let Some((end, _)) = match_placeholder_at(&tokens, i + 1, replacement_id) {
                        parts.push(TemplatePart::ConcatIdent {
                            prefix: ident.clone(),
                        });
                        i = end;
                        continue;
                    }
                }
            }

            if let Some((end, span)) = match_placeholder_at(&tokens, i, replacement_id) {
                parts.push(TemplatePart::Placeholder { span });
                i = end;
                continue;
            }

            match &tokens[i] {
                TokenTree::Group(group) => {
                    parts.push(TemplatePart::Group {
                        delimiter: group.delimiter(),
                        span: group.span(),
                        inner: Template::compile(group.stream(), replacement_id),
                    });
                }
                TokenTree::Literal(lit) => {
                    let rendered = lit.to_string();
                    if rendered.contains(&needle) {
                        parts.push(TemplatePart::LitRewrite {
                            lit: lit.clone(),
                            needle: needle.clone(),
                        });
                    } else {
                        parts.push(TemplatePart::Token(TokenTree::Literal(lit.clone())));
                    }
                }
                other => parts.push(TemplatePart::Token(other.clone())),
            }
            i += 1;
        }

        Template { parts }
    }

    fn instantiate(&self, replacement: &Replacement) -> proc_macro2::TokenStream {
        let mut out = Vec::with_capacity(self.parts.len());
        for part in &self.parts {
            match part {
                TemplatePart::Token(tt) => out.push(tt.clone()),
                TemplatePart::Placeholder { span } => {
                    out.push(replacement.with_span(*span).token);
                }
                TemplatePart::ConcatIdent { prefix } => {
                    let name = format!("{}{}", prefix, replacement.text);
                    out.push(TokenTree::Ident(Ident::new(&name, prefix.span())));
                }
                TemplatePart::Group {
                    delimiter,
                    span,
                    inner,
                } => {
                    let mut group = Group::new(*delimiter, inner.instantiate(replacement));
                    group.set_span(*span);
                    out.push(TokenTree::Group(group));
                }
                TemplatePart::LitRewrite { lit, needle } => {
                    let rendered = lit.to_string();
                    let replaced = rendered.replace(needle, &replacement.text);
                    match replaced.parse::<proc_macro2::TokenStream>() {
                        Ok(stream) => {
                            if let Some(mut tt) = stream.into_iter().next() {
                                tt.set_span(lit.span());
                                out.push(tt);
                            } else {
                                out.push(TokenTree::Literal(lit.clone()));
                            }
                        }
                        Err(_) => out.push(TokenTree::Literal(lit.clone())),
                    }
                }
            }
        }
        out.into_iter().collect()
    }

    /// Approximate token count for capacity hints.
    fn rough_len(&self) -> usize {
        self.parts
            .iter()
            .map(|p| match p {
                TemplatePart::Group { inner, .. } => inner.rough_len() + 2,
                _ => 1,
            })
            .sum()
    }
}

fn push_flat_to_string(flat: &[FlatToken], s: &mut String) {
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
}

fn push_stream_to_string(stream: proc_macro2::TokenStream, s: &mut String) {
    use std::fmt::Write;
    let _ = write!(s, "{}", stream);
}

fn iter_values(start: u32, end: u32, step: usize) -> impl Iterator<Item = u32> {
    (start..=end).step_by(step)
}

impl RepeatedInput {
    fn expand(self) -> Result<proc_macro2::TokenStream> {
        let RepeatedInput {
            prelude,
            definition,
            postlude,
        } = self;

        let replacement_id = definition.replacement_id.to_string();
        let template = Template::compile(definition.repeat_block, &replacement_id);
        let separator = definition.separator.value();
        let sep_span = definition.separator.span();

        let replacements = collect_replacements(&definition.source)?;

        // Empty separator concatenates rendered tokens so adjacent literals/idents
        // can glue (`1` x4 -> `1111`). That path still needs a string round-trip.
        if separator.is_empty() {
            return expand_gluing(&prelude, &postlude, &template, &replacements, sep_span);
        }

        // Fast path: assemble FlatTokens and unflatten. Whitespace-only separators
        // (including the default `"\n"`) are insignificant to the Rust lexer and
        // can be omitted — avoiding TokenStream::to_string + full re-lex.
        let per = template.rough_len().max(1);
        let mut flat = Vec::with_capacity(
            prelude.len() + postlude.len() + replacements.len().saturating_mul(per),
        );
        flat.extend(prelude);

        let sep_flat = if separator.chars().all(char::is_whitespace) {
            None
        } else {
            let sep_stream: proc_macro2::TokenStream =
                separator.parse().map_err(|err: proc_macro2::LexError| {
                    syn::Error::new(
                        sep_span,
                        format!("failed to parse separator tokens: {}", err),
                    )
                })?;
            Some(flatten(sep_stream))
        };

        let mut first = true;
        for replacement in &replacements {
            if !first {
                if let Some(ref sep) = sep_flat {
                    flat.extend(sep.iter().cloned());
                }
            }
            first = false;
            let substituted = template.instantiate(replacement);
            flatten_into(substituted, &mut flat);
        }
        flat.extend(postlude);

        unflatten(&flat)
    }
}

fn collect_replacements(source: &RepeatSource) -> Result<Vec<Replacement>> {
    match source {
        RepeatSource::Range { low, high, step } => {
            let start = low.base10_parse::<u32>()?;
            let end = high.base10_parse::<u32>()?;
            let step_val = match step {
                Some(step) => step.base10_parse::<usize>()?,
                None => 1,
            };
            if step_val == 0 {
                return Err(syn::Error::new(
                    step.as_ref()
                        .map(|s| s.span())
                        .unwrap_or_else(Span::call_site),
                    "repeat step must be non-zero",
                ));
            }
            let span = low.span();
            Ok(iter_values(start, end, step_val)
                .map(|v| Replacement::from_number(v, span))
                .collect())
        }
        RepeatSource::List(values) => Ok(values.iter().map(SubstValue::as_replacement).collect()),
    }
}

fn expand_gluing(
    prelude: &[FlatToken],
    postlude: &[FlatToken],
    template: &Template,
    replacements: &[Replacement],
    error_span: Span,
) -> Result<proc_macro2::TokenStream> {
    let mut buf = String::with_capacity(
        prelude.len().saturating_mul(4)
            + postlude.len().saturating_mul(4)
            + replacements
                .len()
                .saturating_mul(template.rough_len().saturating_mul(8).max(32)),
    );
    push_flat_to_string(prelude, &mut buf);
    for replacement in replacements {
        let substituted = template.instantiate(replacement);
        push_stream_to_string(substituted, &mut buf);
    }
    push_flat_to_string(postlude, &mut buf);

    buf.parse().map_err(|err: proc_macro2::LexError| {
        syn::Error::new(
            error_span,
            format!(
                "failed to parse expanded tokens (empty separator glues output as text): {}",
                err
            ),
        )
    })
}

#[proc_macro]
pub fn repeated(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RepeatedInput);
    input
        .expand()
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
