use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateParseError {
    #[error("Unexpected token {0:?}")]
    Unexpected(Token),
    #[error("Unmatched delimiter {0}")]
    Unmatched(String),
    #[error("Empty block")]
    EmptyBlock,
    #[error("{0}")]
    EvalError(#[from] proc_macro2::LexError),
}

pub struct Template(Part);

impl FromStr for Template {
    type Err = TemplateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut block_stack = vec![Block {
            begin: None,
            body: vec![],
        }];

        for token in Self::tokenize(s)? {
            match token.clone() {
                Token::Literal(literal) => {
                    block_stack
                        .last_mut()
                        .unwrap()
                        .body
                        .push(Part::Literal(literal));
                }
                Token::Eval(code) => {
                    block_stack.last_mut().unwrap().body.push(Part::Eval(code));
                }
                Token::BlockBegin(begin) => {
                    block_stack.push(Block {
                        begin: Some(begin),
                        body: vec![],
                    });
                }
                Token::BlockContinue(begin) => {
                    if block_stack.len() == 1 {
                        return Err(TemplateParseError::Unexpected(token));
                    }

                    let block = block_stack.pop().unwrap();
                    block_stack
                        .last_mut()
                        .unwrap()
                        .body
                        .push(Part::Block(block));

                    block_stack.push(Block {
                        begin: Some(begin),
                        body: vec![],
                    });
                }
                Token::BlockEnd => {
                    if block_stack.len() == 1 {
                        return Err(TemplateParseError::Unexpected(token));
                    }

                    let block = block_stack.pop().unwrap();
                    block_stack
                        .last_mut()
                        .unwrap()
                        .body
                        .push(Part::Block(block));
                }
            }
        }

        Ok(Template(Part::Block(block_stack.pop().unwrap())))
    }
}

impl Template {
    fn next_of<'a>(delims: &[&str], s: &'a str) -> Option<(usize, &'a str)> {
        delims
            .iter()
            .filter_map(|d| s.match_indices(d).next())
            .min_by_key(|&(i, _)| i)
    }

    fn tokenize(mut s: &str) -> Result<Vec<Token>, TemplateParseError> {
        let mut tokens = vec![];

        while let Some((start, open_delim)) = Self::next_of(&["{%", "{{"], s) {
            if start > 0 {
                let mut literal = s[..start].to_string();
                if literal.ends_with('\n') {
                    literal.pop();
                    if literal.ends_with('\r') { literal.pop(); }
                }
                tokens.push(Token::Literal(literal));
                s = &s[start..];
            }

            let (len, close_delim) = Self::next_of(&["%}", "}}"], s)
                .ok_or_else(|| TemplateParseError::Unmatched(open_delim.into()))?;

            let expected_close_delim = match open_delim {
                "{%" => "%}",
                "{{" => "}}",
                _ => unreachable!(),
            };

            if close_delim != expected_close_delim {
                return Err(TemplateParseError::Unmatched(open_delim.into()));
            }

            let inner = s[..len].strip_prefix(open_delim).unwrap();

            match open_delim {
                "{{" => tokens.push(Token::Eval(inner.parse()?)),
                "{%" => {
                    let inner_tokens: TokenStream = inner.parse()?;
                    match inner_tokens
                        .clone()
                        .into_iter()
                        .next()
                        .ok_or(TemplateParseError::EmptyBlock)?
                    {
                        TokenTree::Ident(ident) => match ident.to_string().as_ref() {
                            "end" => tokens.push(Token::BlockEnd),
                            "else" => tokens.push(Token::BlockContinue(inner_tokens)),
                            _ => tokens.push(Token::BlockBegin(inner_tokens)),
                        }
                        _ => tokens.push(Token::BlockBegin(inner_tokens)),
                    }
                }
                _ => unreachable!(),
            }

            s = s[len..].strip_prefix(close_delim).unwrap();
        }

        Ok(tokens)
    }

    pub fn emit_render(self, struct_name: Ident) -> TokenStream {
        let body = self.0.emit_render();

        quote! {
            impl ::templariusz::Template for #struct_name {
                fn render(self) -> String {
                    use ::std::fmt::Write;
                    let mut result = String::new();
                    #body
                    result
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Literal(String),
    Eval(TokenStream),
    BlockBegin(TokenStream),    // if, for, while
    BlockContinue(TokenStream), // else, else if
    BlockEnd,                   // end
}

enum Part {
    Literal(String),
    Eval(TokenStream),
    Block(Block),
}

struct Block {
    begin: Option<TokenStream>,
    body: Vec<Part>,
}

impl Part {
    fn emit_render(self) -> TokenStream {
        match self {
            Self::Literal(lit) => quote! { result.push_str(#lit); },
            Self::Eval(code) => quote! { write!(&mut result, "{}", { #code }).unwrap(); },
            Self::Block(Block { begin, body }) => {
                let inner = body
                    .into_iter()
                    .map(Self::emit_render)
                    .collect::<TokenStream>();

                quote! { #begin { #inner } }
            }
        }
    }
}
