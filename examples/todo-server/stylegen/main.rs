//! Private, host-only generator; never linked into either application target.
#[allow(dead_code)]
mod builder;
#[path = "../../todo-ui/src/style_names.rs"]
#[allow(dead_code)]
mod style_names;
mod styles;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let css = styles::stylesheet()?.css()?;
    print!("{css}{}", include_str!("plain.css"));
    Ok(())
}
