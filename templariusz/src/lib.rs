pub use templariusz_macro::template;

pub trait Template {
    fn render(self) -> String;
}
