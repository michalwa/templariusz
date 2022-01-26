use templariusz::{Template, template};

#[template(path = "basic.template")]
struct Greeting {
    name: String,
    items: Vec<String>,
}

fn main() {
    let greeting = Greeting {
        name: "John Doe".into(),
        items: vec!["Lorem ipsum".into(), "Foobar".into()],
    };
    println!("{}", greeting.render());
}
