use templariusz::{Template, template};

#[template(path = "basic.template")]
struct Greeting {
    name: String,
}

fn main() {
    let greeting = Greeting {
        name: "John Doe".into(),
    };
    println!("{}", greeting.render());
}
