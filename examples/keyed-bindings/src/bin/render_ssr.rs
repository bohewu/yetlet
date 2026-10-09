fn main() {
    match yetlet_keyed_bindings::demo_ssr_html() {
        Ok(html) => print!("{html}"),
        Err(error) => {
            eprintln!("failed to render M7b SSR fixture: {error}");
            std::process::exit(1);
        }
    }
}
