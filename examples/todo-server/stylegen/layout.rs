use crate::builder::{
    Align, Block::*, Color, Declaration::*, Display, Justify, Length::*, Module, Rule,
};
use crate::style_names::*;
pub fn styles() -> Module {
    Module {
        name: "layout",
        blocks: vec![
            Base(Rule::new(
                SHELL,
                vec![
                    LengthToken("--app-space", Rem(1.0)),
                    ColorToken("--app-ink", Color::Hex("#243247")),
                    ColorToken("--app-complete", Color::Hex("#315f4a")),
                    Color(Color::Var("--app-ink")),
                    Padding(Var("--app-space")),
                    FontSize(Rem(1.0)),
                ],
            )),
            Base(Rule::new(
                CONTENT,
                vec![
                    Display(Display::Grid),
                    Columns(1, 1.0),
                    Gap(Var("--app-space")),
                ],
            )),
            MinWidth(Px(768.0), vec![Rule::new(CONTENT, vec![Columns(2, 1.0)])]),
            Base(Rule::new(
                TOOLBAR,
                vec![
                    Display(Display::Flex),
                    AlignItems(Align::Center),
                    JustifyContent(Justify::SpaceBetween),
                    Gap(Rem(0.75)),
                ],
            )),
            Base(Rule::new(PANEL, vec![Padding(Rem(1.25))])),
        ],
    }
}
