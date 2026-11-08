use crate::builder::{Block::Base, Color, Declaration::*, Display, Length::*, Module, Rule};
use crate::style_names::DIALOG_OPEN;

pub fn styles() -> Module {
    Module {
        name: "dialog",
        blocks: vec![Base(Rule::new(
            DIALOG_OPEN,
            vec![
                Display(Display::Block),
                Width(Rem(24.0)),
                Height(Auto),
                Padding(Rem(1.0)),
                Color(Color::Rgb(32, 48, 64)),
            ],
        ))],
    }
}
