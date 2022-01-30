use templariusz::template;

#[template(source = "{{ self.greeting }} {% yield person %}! Today is {% yield date %}.")]
struct Layout {
    greeting: String,
}

#[template(source = "{{ self.first_name }} {{ self.last_name }}")]
struct Person {
    first_name: String,
    last_name: String,
}

#[template(source = "{{ self.day }}/{{ self.month }}")]
struct Date {
    day: u8,
    month: u8,
}

#[test]
fn layout() {
    let layout = Layout { greeting: "Hello".into() };
    let person = Person { first_name: "Jan".into(), last_name: "Kowalski".into() };
    let date = Date { day: 15, month: 7 };

    assert_eq!(layout.render_with(person, date), "Hello Jan Kowalski! Today is 15/7.");
}
