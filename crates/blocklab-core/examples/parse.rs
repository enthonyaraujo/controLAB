//! Experimente o parser pela linha de comando:
//!
//! ```text
//! cargo run --example parse -- "10/s(s+1)" "K/(s+a)" "s^0.5"
//! cargo run --example parse            # modo interativo (Ctrl+D sai)
//! ```

use std::io::{self, BufRead, Write};

use blocklab_core::format::{Mode, to_latex, to_text};
use blocklab_core::parser::parse_rational;

fn show(src: &str) {
    match parse_rational(src) {
        Ok(f) => {
            println!("{src}");
            println!("  simbólico: {}", to_text(&f, Mode::Symbolic));
            println!("  em s:      {}", to_text(&f, Mode::Numeric));
            println!("  LaTeX:     {}", to_latex(&f, Mode::Symbolic));
            let symbols: Vec<String> = f.symbols().iter().map(|s| s.to_string()).collect();
            if !symbols.is_empty() {
                println!("  símbolos: {}", symbols.join(", "));
            }
        }
        Err(e) => {
            let rendered = e.render(src);
            for line in rendered.lines() {
                println!("  {line}");
            }
        }
    }
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        args.iter().for_each(|a| show(a));
        return;
    }
    println!("Digite uma expressão por linha (Ctrl+D para sair).");
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim();
        if !line.is_empty() {
            show(line);
        }
    }
}
