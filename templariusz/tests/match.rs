use templariusz::{template, Template};

#[template(path = "match.template")]
struct Menu {
    items: Vec<MenuItem>,
}

enum MenuItem {
    Pizza { toppings: String },
    Kebab,
}

#[test]
fn match_enum() {
    let menu = Menu {
        items: vec![
            MenuItem::Pizza {
                toppings: "mozarella".into(),
            },
            MenuItem::Kebab,
        ],
    };

    assert_eq!(
        menu.render().lines().collect::<Vec<_>>(),
        &[
            "Menu:",
            "  - Pizza with mozarella",
            "  - Kebab",
        ]
    );
}
