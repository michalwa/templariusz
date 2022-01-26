//! Caller-side library which the macro crate compiles against

pub use templariusz_macro::template;

pub trait Template {
    fn render(self) -> String;
}
