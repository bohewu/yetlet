//! Private, finite CSS vocabulary. Validation happens before returning any CSS.
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub enum Length {
    Px(f64),
    Rem(f64),
    Percent(f64),
    Auto,
    Var(&'static str),
}
#[derive(Clone, Debug)]
pub enum Color {
    Rgb(i32, i32, i32),
    Hex(&'static str),
    Var(&'static str),
}
#[derive(Clone, Debug)]
pub enum Display {
    Block,
    Flex,
    Grid,
    None,
}
#[derive(Clone, Debug)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}
#[derive(Clone, Debug)]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
#[derive(Clone, Debug)]
pub enum Weight {
    Normal,
    Bold,
}
#[derive(Clone, Debug)]
pub enum Declaration {
    Display(Display),
    AlignItems(Align),
    JustifyContent(Justify),
    Gap(Length),
    Padding(Length),
    Width(Length),
    Height(Length),
    Columns(u32, f64),
    FontSize(Length),
    FontWeight(Weight),
    Color(Color),
    LengthToken(&'static str, Length),
    ColorToken(&'static str, Color),
}
#[derive(Clone, Debug)]
pub struct Rule {
    pub class: &'static str,
    pub declarations: Vec<Declaration>,
}
impl Rule {
    pub fn new(class: &'static str, declarations: Vec<Declaration>) -> Self {
        Self {
            class,
            declarations,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Block {
    Base(Rule),
    MinWidth(Length, Vec<Rule>),
}
#[derive(Clone, Debug)]
pub struct Module {
    pub name: &'static str,
    pub blocks: Vec<Block>,
}
#[derive(Debug)]
pub struct StyleError(String);
impl std::fmt::Display for StyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for StyleError {}

fn class_valid(name: &str) -> bool {
    let mut chars = name.bytes();
    (1..=128).contains(&name.len())
        && chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}
fn token(name: &str) -> Result<&str, String> {
    let suffix = name
        .strip_prefix("--app-")
        .ok_or("expected --app-* custom property")?;
    if class_valid(suffix) {
        Ok(name)
    } else {
        Err("invalid custom property name".into())
    }
}
fn number(value: f64) -> Result<String, String> {
    if value.is_finite() && value >= 0.0 {
        Ok(if value == 0.0 {
            "0".into()
        } else {
            value.to_string()
        })
    } else {
        Err("expected finite nonnegative number".into())
    }
}
fn length(value: &Length, auto: bool, reference: bool, percent: bool) -> Result<String, String> {
    match value {
        Length::Px(n) => Ok(format!("{}px", number(*n)?)),
        Length::Rem(n) => Ok(format!("{}rem", number(*n)?)),
        Length::Percent(n) if percent => Ok(format!("{}%", number(*n)?)),
        Length::Auto if auto => Ok("auto".into()),
        Length::Var(name) if reference => Ok(format!("var({})", token(name)?)),
        _ => Err("unsupported length for property/context".into()),
    }
}
fn color(value: &Color) -> Result<String, String> {
    match value {
        Color::Rgb(r, g, b) if [r, g, b].into_iter().all(|v| (0..=255).contains(v)) => {
            Ok(format!("rgb({r}, {g}, {b})"))
        }
        Color::Hex(hex)
            if (hex.len() == 4 || hex.len() == 7)
                && hex.starts_with('#')
                && hex[1..].bytes().all(|c| c.is_ascii_hexdigit()) =>
        {
            Ok(hex.to_string())
        }
        Color::Var(name) => Ok(format!("var({})", token(name)?)),
        _ => Err("expected RGB channels 0..255 or #RGB/#RRGGBB".into()),
    }
}
impl Declaration {
    fn property(&self) -> &str {
        match self {
            Self::Display(_) => "display",
            Self::AlignItems(_) => "align-items",
            Self::JustifyContent(_) => "justify-content",
            Self::Gap(_) => "gap",
            Self::Padding(_) => "padding",
            Self::Width(_) => "width",
            Self::Height(_) => "height",
            Self::Columns(..) => "grid-template-columns",
            Self::FontSize(_) => "font-size",
            Self::FontWeight(_) => "font-weight",
            Self::Color(_) => "color",
            Self::LengthToken(name, _) | Self::ColorToken(name, _) => name,
        }
    }
    fn value(&self) -> Result<String, String> {
        Ok(match self {
            Self::Display(v) => match v {
                Display::Block => "block",
                Display::Flex => "flex",
                Display::Grid => "grid",
                Display::None => "none",
            }
            .into(),
            Self::AlignItems(v) => match v {
                Align::Start => "start",
                Align::Center => "center",
                Align::End => "end",
                Align::Stretch => "stretch",
            }
            .into(),
            Self::JustifyContent(v) => match v {
                Justify::Start => "start",
                Justify::Center => "center",
                Justify::End => "end",
                Justify::SpaceBetween => "space-between",
                Justify::SpaceAround => "space-around",
                Justify::SpaceEvenly => "space-evenly",
            }
            .into(),
            Self::Gap(v) | Self::Padding(v) | Self::FontSize(v) => length(v, false, true, true)?,
            Self::Width(v) | Self::Height(v) => length(v, true, true, true)?,
            Self::Columns(count, fraction) => {
                if *count == 0 || !fraction.is_finite() || *fraction <= 0.0 {
                    return Err("expected positive repeat count and finite positive fr".into());
                }
                format!("repeat({count}, {fraction}fr)")
            }
            Self::FontWeight(v) => match v {
                Weight::Normal => "normal",
                Weight::Bold => "bold",
            }
            .into(),
            Self::Color(v) => color(v)?,
            Self::LengthToken(name, v) => {
                token(name)?;
                length(v, false, false, true)?
            }
            Self::ColorToken(name, v) => {
                token(name)?;
                if matches!(v, Color::Var(_)) {
                    return Err("token value must be a literal color".into());
                }
                color(v)?
            }
        })
    }
}

pub struct StyleSheet {
    modules: Vec<Module>,
}
impl StyleSheet {
    pub fn compose(modules: Vec<Module>) -> Result<Self, StyleError> {
        let mut names = HashMap::new();
        for (position, module) in modules.iter().enumerate() {
            if let Some(previous) = names.insert(module.name, position) {
                return Err(StyleError(format!(
                    "module {}: duplicate registration at {previous} and {position}",
                    module.name
                )));
            }
        }
        Ok(Self { modules })
    }
    pub fn css(&self) -> Result<String, StyleError> {
        let mut output = String::new();
        let mut owners = HashMap::new();
        let mut contexts = HashSet::new();
        for module in &self.modules {
            for block in &module.blocks {
                let (context, rules, indent) = match block {
                    Block::Base(rule) => ("base".to_string(), std::slice::from_ref(rule), ""),
                    Block::MinWidth(width, rules) => {
                        let width = length(width, false, false, false).map_err(|e| {
                            StyleError(format!(
                                "module {} context min-width property min-width: {e}",
                                module.name
                            ))
                        })?;
                        let context = format!("min-width: {width}");
                        output.push_str(&format!("@media ({context}) {{\n"));
                        (context, rules.as_slice(), "  ")
                    }
                };
                for rule in rules {
                    let diagnostic = |property: &str, message: String| {
                        StyleError(format!(
                            "module {} class {} context {} property {}: {}",
                            module.name, rule.class, context, property, message
                        ))
                    };
                    if !class_valid(rule.class) {
                        return Err(diagnostic("class", "invalid generated class token".into()));
                    }
                    if let Some(owner) = owners.insert(rule.class, module.name) {
                        if owner != module.name {
                            return Err(diagnostic(
                                "ownership",
                                format!("class collision between {owner} and {}", module.name),
                            ));
                        }
                    }
                    if !contexts.insert((module.name, rule.class, context.clone())) {
                        return Err(diagnostic("rule", "repeated class/context".into()));
                    }
                    output.push_str(&format!("{indent}.{} {{\n", rule.class));
                    for declaration in &rule.declarations {
                        let value = declaration
                            .value()
                            .map_err(|e| diagnostic(declaration.property(), e))?;
                        output
                            .push_str(&format!("{indent}  {}: {value};\n", declaration.property()));
                    }
                    output.push_str(&format!("{indent}}}\n"));
                }
                if matches!(block, Block::MinWidth(..)) {
                    output.push_str("}\n");
                }
            }
        }
        if output.is_empty() {
            output.push('\n');
        }
        Ok(output)
    }
}
