use crate::builder::{
    Align, Block::*, Color, Declaration::*, Display, Justify, Length::*, Module, Rule, Weight,
};
use crate::style_names::*;

pub fn styles() -> Module {
    Module {
        name: "layout",
        blocks: vec![
            Base(Rule::new(
                SHELL,
                vec![
                    LengthToken("--app-space", Rem(0.75)),
                    ColorToken("--app-ink", Color::Hex("#243247")),
                    Display(Display::Block),
                    Padding(Var("--app-space")),
                    Width(Percent(100.0)),
                    Color(Color::Var("--app-ink")),
                    FontSize(Rem(1.0)),
                    FontWeight(Weight::Normal),
                ],
            )),
            Base(Rule::new(
                TOOLBAR,
                vec![
                    Display(Display::Flex),
                    AlignItems(Align::Center),
                    JustifyContent(Justify::SpaceBetween),
                    Gap(Var("--app-space")),
                    Padding(Px(12.0)),
                    Height(Auto),
                    FontWeight(Weight::Bold),
                ],
            )),
            Base(Rule::new(
                CONTENT,
                vec![Display(Display::Grid), Columns(1, 1.0), Gap(Rem(0.75))],
            )),
            MinWidth(
                Px(768.0),
                vec![Rule::new(CONTENT, vec![Columns(3, 1.0), Gap(Rem(1.0))])],
            ),
        ],
    }
}
