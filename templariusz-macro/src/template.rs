use crate::utils::FindAny;
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
            kind: BlockKind::Block,
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
                        kind: BlockKind::Block,
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
                        kind: BlockKind::Block,
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
                Token::MatchArmBegin(pattern) => {
                    block_stack.push(Block {
                        kind: BlockKind::MatchArm,
                        begin: Some(pattern),
                        body: vec![],
                    });
                }
            }
        }

        Ok(Template(Part::Block(block_stack.pop().unwrap())))
    }
}

impl Template {
    /// Trims up to a single trailing newline and pushes a `Token::Literal`
    /// into the `Vec`
    fn push_literal(tokens: &mut Vec<Token>, literal: impl Into<String>) {
        let mut literal = literal.into();

        if literal.ends_with('\n') {
            literal.pop();
            if literal.ends_with('\r') {
                literal.pop();
            }
        }

        if !literal.is_empty() {
            tokens.push(Token::Literal(literal));
        }
    }

    fn tokenize(mut s: &str) -> Result<Vec<Token>, TemplateParseError> {
        let mut tokens = vec![];

        while let Some((start, open_delim)) = s.find_any(&["{%", "{{"]) {
            if start > 0 {
                Self::push_literal(&mut tokens, &s[..start]);
                s = &s[start..];
            }

            let (len, close_delim) = s
                .find_any(&["%}", "}}"])
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
                    let mut left = inner_tokens.clone().into_iter();

                    match left.next().ok_or(TemplateParseError::EmptyBlock)? {
                        TokenTree::Ident(ident) => match ident.to_string().as_ref() {
                            "end" => tokens.push(Token::BlockEnd),
                            "else" => tokens.push(Token::BlockContinue(inner_tokens)),
                            "case" => tokens.push(Token::MatchArmBegin(left.collect())),
                            _ => tokens.push(Token::BlockBegin(inner_tokens)),
                        },
                        _ => tokens.push(Token::BlockBegin(inner_tokens)),
                    }
                }
                _ => unreachable!(),
            }

            s = s[len..].strip_prefix(close_delim).unwrap();
        }

        if !s.is_empty() {
            Self::push_literal(&mut tokens, s);
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
    MatchArmBegin(TokenStream),
}

#[derive(Debug)]
enum Part {
    Literal(String),
    Eval(TokenStream),
    Block(Block),
}

#[derive(Debug)]
struct Block {
    kind: BlockKind,
    begin: Option<TokenStream>,
    body: Vec<Part>,
}

#[derive(Debug)]
enum BlockKind {
    Block,
    MatchArm,
}

impl Part {
    fn emit_render(self) -> TokenStream {
        match self {
            Self::Literal(lit) => quote! { result.push_str(#lit); },
            Self::Eval(code) => quote! { write!(&mut result, "{}", { #code }).unwrap(); },
            Self::Block(Block {
                kind: BlockKind::Block,
                begin,
                body,
            }) => {
                let inner = body
                    .into_iter()
                    .map(Self::emit_render)
                    .collect::<TokenStream>();

                quote! { #begin { #inner } }
            }
            Self::Block(Block {
                kind: BlockKind::MatchArm,
                begin: pattern,
                body,
            }) => {
                let inner = body
                    .into_iter()
                    .map(Self::emit_render)
                    .collect::<TokenStream>();

                quote! { #pattern => { #inner } }
            }
        }
    }
}
