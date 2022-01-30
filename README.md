# templariusz

Compiled string templates for Rust

## Usage

See [tests](templariusz/tests) for detailed usage examples

### Basic example

```rs
use templariusz::{template, Template};

#[template(path = "basic.template")]
struct Greeting {
    name: String,
    items: Vec<String>,
    numbers: Vec<u32>,
    morning: bool,
    evening: bool,
}

fn main() {
    let greeting = Greeting {
        name: "John Doe".into(),
        items: vec!["Lorem ipsum".into(), "Foobar".into()],
        numbers: vec![42, 30, 7, 15],
        morning: false,
        evening: true,
    };
    println!("{}", greeting.render());
}
```

`basic.template`:
```
Hello, {{ self.name }}!
Items:
{% for item in self.items %}
    - {{ item }}
{% end %}
Numbers: {% for number in self.numbers %}{{ number }}, {% end %}

{% if self.morning %}
    Good morning
{% else if self.evening %}
    Good evening
{% end %}
```

Templates are compiled directly into a `render(self)` function, so you can insert Rust pretty much anywhere and it will just be inlined.
- `{{ }}` compile to `write!("{}")` calls, so you can use them with anything implementing `Display`.
- `{% %}` begin or end blocks, which are compiled to regular Rust blocks, so you can use `for`-s, `while`-s and `if/else`-s. `match` uses a special `{% case ... %}` syntax for match arms instead of `=>`.

## Compiling

Requires nightly
