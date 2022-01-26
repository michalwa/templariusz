use templariusz::{Template, template};

#[template("Hello, {{ self.name }}!")]
struct Greeting {
    name: String,
}

fn main() {
    let greeting = Greeting {
        name: "John Doe".into(),
    };
    println!("{}", greeting.render());
}
