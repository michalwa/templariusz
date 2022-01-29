use templariusz::{template, Template};

#[template(source = "{{ self.first_name }} {{ self.last_name }}")]
struct Person {
    first_name: String,
    last_name: String,
}

#[template(path = "basic.template")]
struct Greeting {
    name: String,
    items: Vec<String>,
    numbers: Vec<u32>,
    morning: bool,
    evening: bool,
}

#[test]
fn from_source() {
    let person = Person {
        first_name: "John".into(),
        last_name: "Doe".into(),
    };
    assert_eq!(person.render(), "John Doe");
}

#[test]
fn from_file() {
    let greeting = Greeting {
        name: "John Doe".into(),
        items: vec!["Lorem ipsum".into(), "Foobar".into()],
        numbers: vec![42, 30, 7, 15],
        morning: false,
        evening: true,
    };
    assert_eq!(
        greeting.render().lines().collect::<Vec<_>>(),
        [
            "Hello, John Doe!",
            "Items:",
            "    - Lorem ipsum",
            "    - Foobar",
            "Numbers: 42, 30, 7, 15, ",
            "",
            "    Good evening",
            "",
            "Bye!",
        ],
    )
}
