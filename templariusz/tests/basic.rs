use templariusz::{Template, template};

#[template(source = "{{ self.first_name }} {{ self.last_name }}")]
struct Person {
    first_name: String,
    last_name: String,
}

#[test]
fn basic_substitution() {
    let person = Person {
        first_name: "John".into(),
        last_name: "Doe".into(),
    };
    assert_eq!(person.render(), "John Doe");
}
