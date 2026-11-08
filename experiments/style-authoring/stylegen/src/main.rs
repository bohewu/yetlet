use std::io::{self, Write};

fn main() {
    let result = app_stylegen::styles::stylesheet()
        .and_then(|sheet| sheet.css())
        .map_err(|error| error.to_string())
        .and_then(|css| {
            io::stdout()
                .lock()
                .write_all(css.as_bytes())
                .map_err(|error| error.to_string())
        });
    if let Err(error) = result {
        eprintln!("stylegen: {error}");
        std::process::exit(1);
    }
}
