use Declaration::*;
use Length::*;
use app_stylegen::builder::*;
use app_stylegen::builder::{Color, Display};
use app_stylegen::{style_names, styles};

fn module(name: &'static str, blocks: Vec<Block>) -> Module {
    Module { name, blocks }
}
fn rule(class: &'static str, values: Vec<Declaration>) -> Block {
    Block::Base(Rule::new(class, values))
}
fn css(modules: Vec<Module>) -> Result<String, StyleError> {
    StyleSheet::compose(modules)?.css()
}
fn error(values: Vec<Declaration>) -> String {
    css(vec![module("layout", vec![rule("app-x", values)])])
        .unwrap_err()
        .to_string()
}

#[test]
fn complete_default_registry_matches_plain_css_without_views() {
    let actual = styles::stylesheet().unwrap().css().unwrap();
    assert_eq!(actual, include_str!("../../assets/plain.css"));
    assert!(actual.contains(&format!(".{} {{", style_names::DIALOG_OPEN)));
    assert_eq!(actual, styles::stylesheet().unwrap().css().unwrap());
    assert!(!actual.contains('\r'));
    assert!(actual.ends_with("}\n"));
}
#[test]
fn duplicate_modules_even_equal_identify_both_positions() {
    let m = module("layout", vec![]);
    let e = css(vec![m.clone(), m]).unwrap_err().to_string();
    assert!(e.contains("layout") && e.contains("0 and 1"), "{e}");
}
#[test]
fn equal_class_different_owner_is_collision_in_any_context() {
    let e = css(vec![
        module("layout", vec![rule("app-x", vec![Width(Auto)])]),
        module(
            "dialog",
            vec![Block::MinWidth(
                Px(10.0),
                vec![Rule::new("app-x", vec![Width(Auto)])],
            )],
        ),
    ])
    .unwrap_err()
    .to_string();
    for part in ["layout", "dialog", "app-x", "min-width", "collision"] {
        assert!(e.contains(part), "{e}");
    }
}
#[test]
fn same_owner_base_and_media_allowed_repeated_context_rejected() {
    let base = rule("app-x", vec![Gap(Px(1.0))]);
    let media = Block::MinWidth(Px(768.0), vec![Rule::new("app-x", vec![Gap(Rem(1.0))])]);
    assert!(css(vec![module("layout", vec![base.clone(), media.clone()])]).is_ok());
    for blocks in [vec![base.clone(), base], vec![media.clone(), media]] {
        assert!(
            css(vec![module("layout", blocks)])
                .unwrap_err()
                .to_string()
                .contains("repeated class/context")
        );
    }
}
#[test]
fn explicit_module_media_rule_and_duplicate_declaration_order() {
    let text = css(vec![
        module(
            "z",
            vec![
                rule(
                    "z",
                    vec![
                        Display(Display::Flex),
                        Display(Display::Grid),
                        Padding(Px(1.0)),
                        Padding(Rem(2.0)),
                    ],
                ),
                Block::MinWidth(
                    Px(900.0),
                    vec![
                        Rule::new("z", vec![Gap(Px(1.0)), Gap(Px(2.0))]),
                        Rule::new("y", vec![]),
                    ],
                ),
                Block::MinWidth(Px(100.0), vec![Rule::new("z", vec![Width(Auto)])]),
            ],
        ),
        module("a", vec![rule("a", vec![])]),
    ])
    .unwrap();
    assert_eq!(
        text,
        ".z {\n  display: flex;\n  display: grid;\n  padding: 1px;\n  padding: 2rem;\n}\n@media (min-width: 900px) {\n  .z {\n    gap: 1px;\n    gap: 2px;\n  }\n  .y {\n  }\n}\n@media (min-width: 100px) {\n  .z {\n    width: auto;\n  }\n}\n.a {\n}\n"
    );
}
#[test]
fn invalid_values_have_property_and_owner_context() {
    for (declaration, property) in [
        (Gap(Px(f64::NAN)), "gap"),
        (Padding(Rem(f64::INFINITY)), "padding"),
        (Width(Percent(-1.0)), "width"),
        (Height(Px(-1.0)), "height"),
        (Gap(Auto), "gap"),
        (Padding(Auto), "padding"),
        (FontSize(Auto), "font-size"),
        (Columns(0, 1.0), "grid-template-columns"),
        (Columns(2, 0.0), "grid-template-columns"),
        (Columns(2, -1.0), "grid-template-columns"),
        (Columns(2, f64::INFINITY), "grid-template-columns"),
        (Color(Color::Rgb(256, 0, 0)), "color"),
        (Color(Color::Rgb(-1, 0, 0)), "color"),
        (Color(Color::Hex("#ggg")), "color"),
        (Color(Color::Hex("red;} body{")), "color"),
        (LengthToken("--evil", Px(1.0)), "--evil"),
        (LengthToken("--app-x;", Px(1.0)), "--app-x;"),
        (LengthToken("--app-x", Auto), "--app-x"),
        (ColorToken("--app-c", Color::Var("--app-other")), "--app-c"),
        (Gap(Var("--app-x}")), "gap"),
        (Color(Color::Var("--app-")), "color"),
    ] {
        let e = error(vec![Width(Px(1.0)), declaration]);
        for part in ["layout", "app-x", "base", property] {
            assert!(e.contains(part), "{e}");
        }
    }
}
#[test]
fn class_grammar_and_media_reject_unsupported_inputs() {
    for name in [
        "",
        "9panel",
        "app-toolbar active",
        "x{color:red}",
        "é",
        &"x".repeat(129),
    ] {
        // Names in normal definitions are static; leak only bounded test strings.
        let name = Box::leak(name.to_string().into_boxed_str());
        assert!(css(vec![module("layout", vec![rule(name, vec![])])]).is_err());
    }
    for width in [
        Auto,
        Percent(50.0),
        Var("--app-break"),
        Px(-1.0),
        Rem(f64::NAN),
    ] {
        let e = css(vec![module("layout", vec![Block::MinWidth(width, vec![])])])
            .unwrap_err()
            .to_string();
        assert!(
            e.contains("layout context min-width property min-width"),
            "{e}"
        );
    }
}
#[test]
fn remaining_finite_vocabulary_and_boundary_values() {
    let declarations = vec![
        Display(Display::None),
        AlignItems(Align::Start),
        AlignItems(Align::End),
        AlignItems(Align::Stretch),
        JustifyContent(Justify::Start),
        JustifyContent(Justify::Center),
        JustifyContent(Justify::End),
        JustifyContent(Justify::SpaceAround),
        JustifyContent(Justify::SpaceEvenly),
        Gap(Px(-0.0)),
        Padding(Percent(5.0)),
        FontSize(Px(12.0)),
        Color(Color::Hex("#AbC")),
        ColorToken("--app-C_2", Color::Rgb(0, 255, 0)),
    ];
    let text = css(vec![module("layout", vec![rule("_X-2", declarations)])]).unwrap();
    assert!(
        text.contains("gap: 0px;")
            && text.contains("color: #AbC;")
            && text.contains("--app-C_2: rgb(0, 255, 0);")
    );
}
