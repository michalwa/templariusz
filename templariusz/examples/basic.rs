use templariusz::{Template, template};

#[template(path = "basic.template")]
struct Greeting {
    name: String,
    items: Vec<String>,
    numbers: Vec<u32>,
}

fn main() {
    let greeting = Greeting {
        name: "John Doe".into(),
        items: vec!["Lorem ipsum".into(), "Foobar".into()],
        numbers: vec![42, 30, 7, 15],
    };
    println!("{}", greeting.render());
}
