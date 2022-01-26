#![feature(proc_macro_span)]

use std::{str::FromStr, fs};
use proc_macro2::{TokenStream};
use syn::{self, AttributeArgs, ItemStruct, Ident, parse_macro_input};
use quote::quote;
use darling::FromMeta;
use thiserror::Error;

#[derive(Debug, Error)]
enum TemplateParseError {
    #[error("Unterminated eval block `{{'")]
    UnterminatedEval,
    #[error("{0}")]
    EvalError(#[from] proc_macro2::LexError),
}

enum TemplatePart {
    Literal(String),
    Eval(TokenStream),
}

struct Template {
    parts: Vec<TemplatePart>,
}

impl FromStr for Template {
    type Err = TemplateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = vec![];

        let outer = if let Some(end) = s.find("{{") { &s[..end] } else { s };
        parts.push(TemplatePart::Literal(outer.into()));

        for (start, _) in s.match_indices("{{") {
            let len = s[start..].find("}}").ok_or(TemplateParseError::UnterminatedEval)?;
            let inner = &s[start..(start + len)]
                .trim_start_matches("{{")
                .trim_end_matches("}}");

            parts.push(TemplatePart::Eval(inner.parse()?));
        }

        let outer = if let Some(start) = s.rfind("}}") { &s[(start + 2)..] } else { s };
        parts.push(TemplatePart::Literal(outer.into()));

        Ok(Template { parts })
    }
}

impl Template {
    fn emit_render(self, struct_name: Ident) -> TokenStream {
        let mut body = TokenStream::new();

        for part in self.parts {
            body.extend(match part {
                TemplatePart::Literal(lit) => quote! { result.push_str(#lit); },
                TemplatePart::Eval(code) => quote! { result.push_str(&{ #code }); },
            });
        }

        quote! {
            impl templariusz::Template for #struct_name {
                fn render(self) -> String {
                    let mut result = String::new();
                    #body
                    result
                }
            }
        }
    }
}

#[derive(Debug, FromMeta)]
enum MacroArgs {
    Source(String),
    Path(String),
}

#[proc_macro_attribute]
pub fn template(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);
    let mut result = item.clone();

    let source_file = proc_macro::Span::call_site().source_file();

    let args = parse_macro_input!(attr as AttributeArgs);
    let template_struct: ItemStruct = syn::parse2(item).unwrap();

    let args = MacroArgs::from_list(&args).unwrap();

    let source = match args {
        MacroArgs::Source(source) => source,
        MacroArgs::Path(path) => {
            let path = source_file.path().parent().unwrap().join(path);
            fs::read_to_string(path).unwrap()
        }
    };

    let template: Template = source.parse().unwrap();
    result.extend(template.emit_render(template_struct.ident));

    result.into()
}
