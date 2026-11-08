use crate::builder::{Align, Block::*, Declaration::*, Display, Justify, Length::*, Module, Rule};
use crate::style_names::*;

pub fn styles() -> Module {
    Module {
        name: "layout",
        blocks: vec![
            Base(Rule::new(
                TOOLBAR,
                vec![
                    Display(Display::Flex),
                    AlignItems(Align::Center),
                    JustifyContent(Justify::SpaceBetween),
                    Gap(Var("--app-space")),
                ],
            )),
            Base(Rule::new(
                GRID,
                vec![
                    Display(Display::Grid),
                    Columns(1, 1.0),
                    Gap(Var("--app-space")),
                ],
            )),
            MinWidth(Px(768.0), vec![Rule::new(GRID, vec![Columns(3, 1.0)])]),
            Base(Rule::new(
                SPACING,
                vec![
                    Padding(Var("--app-space")),
                    Gap(Var("--app-space")),
                    Display(Display::Flex),
                ],
            )),
        ],
    }
}
