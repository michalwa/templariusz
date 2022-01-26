use std::str::FromStr;
use proc_macro2::TokenStream;
use syn::{self, ItemStruct, LitStr, Ident};
use quote::quote;
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

#[proc_macro_attribute]
pub fn template(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = TokenStream::from(attr);
    let item = TokenStream::from(item);

    let mut result = item.clone();

    let template_lit: LitStr = syn::parse2(attr).unwrap();
    let template_struct: ItemStruct = syn::parse2(item).unwrap();

    let template: Template = template_lit.value().parse().unwrap();
    result.extend(template.emit_render(template_struct.ident));

    result.into()
}
