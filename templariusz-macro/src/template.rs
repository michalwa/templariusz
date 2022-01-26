use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateParseError {
    #[error("Unmatched opening delimiter {0}")]
    UnmatchedOpenDelim(String),
    #[error("Unmatched `end'")]
    UnmatchedEnd,
    #[error("{0}")]
    EvalError(#[from] proc_macro2::LexError),
}

pub struct Template(Part);

impl FromStr for Template {
    type Err = TemplateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        enum Token {
            Literal(String),
            Eval(TokenStream),
            BlockBegin(TokenStream),
            BlockEnd,
        }

        let mut match_indices = s
            .match_indices("{%")
            .chain(s.match_indices("{{"))
            .collect::<Vec<_>>();

        match_indices.sort_by_key(|&(i, _)| i);

        let mut tokens = vec![];
        let mut last_end = 0;

        for (start, open_delim) in match_indices {
            if start > last_end {
                let mut literal = s[last_end..start].to_string();
                if literal.ends_with('\n') {
                    literal.pop();
                    if literal.ends_with('\r') { literal.pop(); }
                }
                tokens.push(Token::Literal(literal));
            }

            let close_delim = match open_delim {
                "{%" => "%}",
                "{{" => "}}",
                _ => unreachable!(),
            };

            let len = s[start..]
                .find(close_delim)
                .ok_or(TemplateParseError::UnmatchedOpenDelim(open_delim.into()))?;
            let end = start + len;
            let inner = &s[(start + 2)..end];

            match open_delim {
                "{%" => {
                    let keyword = inner.trim();
                    if keyword == "end" {
                        tokens.push(Token::BlockEnd);
                    } else if keyword == "else" {
                        tokens.push(Token::BlockEnd);
                        tokens.push(Token::BlockBegin(inner.parse()?));
                    } else {
                        tokens.push(Token::BlockBegin(inner.parse()?));
                    }
                }
                "{{" => tokens.push(Token::Eval(inner.parse()?)),
                _ => unreachable!(),
            }

            last_end = end + 2;
        }

        if last_end < s.len() {
            tokens.push(Token::Literal(s[last_end..].into()));
        }

        let mut block_stack = vec![Block {
            begin: TokenStream::new(),
            body: vec![],
        }];

        for token in tokens {
            match token {
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
                        begin,
                        body: vec![],
                    });
                }
                Token::BlockEnd => {
                    if block_stack.len() == 1 {
                        return Err(TemplateParseError::UnmatchedEnd);
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

enum Part {
    Literal(String),
    Eval(TokenStream),
    Block(Block),
}

struct Block {
    begin: TokenStream,
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
