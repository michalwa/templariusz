#![feature(proc_macro_span)]

use std::fs;
use proc_macro2::TokenStream;
use syn::{self, AttributeArgs, ItemStruct, parse_macro_input};
use darling::FromMeta;
use crate::template::Template;

mod template;
mod utils;

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
