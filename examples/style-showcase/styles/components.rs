use crate::builder::{Block::*, Color, Declaration::*, Length::*, Module, Rule, Weight};
use crate::style_names::*;

pub fn styles() -> Module {
    Module {
        name: "components",
        blocks: vec![
            Base(Rule::new(
                CARD,
                vec![Padding(Var("--app-space")), Color(Color::Var("--app-ink"))],
            )),
            Base(Rule::new(
                BUTTON,
                vec![
                    Padding(Var("--app-control")),
                    FontSize(Rem(0.875)),
                    FontWeight(Weight::Bold),
                ],
            )),
            Base(Rule::new(
                BADGE,
                vec![FontSize(Rem(0.75)), Color(Color::Var("--app-accent"))],
            )),
        ],
    }
}
