use crate::builder::{Block::*, Color, Declaration::*, Length::*, Module, Rule};
use crate::style_names::*;

pub fn styles() -> Module {
    Module {
        name: "tokens",
        blocks: vec![
            Base(Rule::new(
                LIGHT,
                vec![
                    ColorToken("--app-bg", Color::Hex("#f5f7fa")),
                    ColorToken("--app-surface", Color::Hex("#ffffff")),
                    ColorToken("--app-ink", Color::Hex("#17243b")),
                    ColorToken("--app-muted", Color::Hex("#53627a")),
                    ColorToken("--app-border", Color::Hex("#c6cfdb")),
                    ColorToken("--app-accent", Color::Hex("#2458bc")),
                    ColorToken("--app-on-accent", Color::Hex("#ffffff")),
                    ColorToken("--app-tint", Color::Hex("#eaf0ff")),
                    ColorToken("--app-focus", Color::Hex("#874400")),
                ],
            )),
            Base(Rule::new(
                DARK,
                vec![
                    ColorToken("--app-bg", Color::Hex("#111a2a")),
                    ColorToken("--app-surface", Color::Hex("#1a273b")),
                    ColorToken("--app-ink", Color::Hex("#edf2fb")),
                    ColorToken("--app-muted", Color::Hex("#bac7dc")),
                    ColorToken("--app-border", Color::Hex("#65758e")),
                    ColorToken("--app-accent", Color::Hex("#a5c5ff")),
                    ColorToken("--app-on-accent", Color::Hex("#111a2a")),
                    ColorToken("--app-tint", Color::Hex("#263d62")),
                    ColorToken("--app-focus", Color::Hex("#ffc47d")),
                ],
            )),
            Base(Rule::new(
                COMFORTABLE,
                vec![
                    LengthToken("--app-space", Rem(1.25)),
                    LengthToken("--app-control", Rem(0.75)),
                ],
            )),
            Base(Rule::new(
                COMPACT,
                vec![
                    LengthToken("--app-space", Rem(0.75)),
                    LengthToken("--app-control", Rem(0.5)),
                ],
            )),
        ],
    }
}
