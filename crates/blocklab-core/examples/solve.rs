//! Resolve um diagrama e mostra caminhos, laços, Δ, Δₖ e T(s).
//!
//! ```text
//! cargo run --example solve -- --list          # exemplos embutidos
//! cargo run --example solve -- 6               # resolve o exemplo 6
//! cargo run --example solve -- arquivo.json    # resolve um arquivo
//! cargo run --example solve -- --json 6        # imprime o JSON do exemplo
//! ```

use blocklab_core::diagram::{Diagram, examples};
use blocklab_core::format::{Mode, pretty, to_latex, to_text};
use blocklab_core::{SolveError, solve};

fn load(arg: &str) -> Result<(String, Diagram), String> {
    let builtin = examples::all();
    if let Ok(i) = arg.parse::<usize>() {
        return builtin
            .into_iter()
            .nth(i.wrapping_sub(1))
            .map(|(name, d)| (name.to_string(), d))
            .ok_or_else(|| format!("não há exemplo {i}; use --list"));
    }
    let text = std::fs::read_to_string(arg).map_err(|e| format!("{arg}: {e}"))?;
    let d = Diagram::from_json(&text).map_err(|e| format!("{arg}: {e}"))?;
    Ok((arg.to_string(), d))
}

fn run(name: &str, d: &Diagram) {
    println!("== {name}\n");
    let r = match solve(d) {
        Ok(r) => r,
        Err(SolveError::Invalid(errs)) => {
            println!("Diagrama inválido:");
            for e in errs {
                println!("  • {}", e.describe(d));
            }
            return;
        }
        Err(e) => {
            println!("Erro: {e}");
            return;
        }
    };
    println!("Caminhos diretos:");
    for (k, p) in r.paths.iter().enumerate() {
        println!(
            "  P{} = {}    [{}]",
            k + 1,
            p.gain,
            p.describe(&r.sfg, false)
        );
    }
    println!("\nLaços:");
    for (i, l) in r.loops.iter().enumerate() {
        println!(
            "  L{} = {}    [{}]",
            i + 1,
            l.gain,
            l.describe(&r.sfg, true)
        );
    }
    if !r.nontouching.is_empty() {
        println!("\nLaços que não se tocam:");
        for c in &r.nontouching {
            let names: Vec<String> = c.iter().map(|i| format!("L{}", i + 1)).collect();
            println!("  {}", names.join("·"));
        }
    }
    println!("\nΔ = {}", to_text(&r.delta, Mode::Symbolic));
    for (k, dk) in r.cofactors.iter().enumerate() {
        println!("Δ{} = {}", k + 1, to_text(dk, Mode::Symbolic));
    }
    let t = &r.transfer;
    println!("\nT(s) = Y/X");
    let sym = pretty(t, Mode::Symbolic);
    println!("  simbólico: {}", sym.text());
    if t.num().vars().contains(&blocklab_core::algebra::Var::S)
        || t.den().vars().contains(&blocklab_core::algebra::Var::S)
    {
        println!("  em s:      {}", to_text(t, Mode::Numeric));
    }
    println!("  ASCII:     {}", sym.ascii());
    println!("  LaTeX:     {}", to_latex(t, Mode::Symbolic));

    match d.numeric_params() {
        Ok(p) if !p.is_empty() => match t.substitute(&p) {
            Ok(n) => println!("  numérico:  {}", to_text(&n, Mode::Numeric)),
            Err(e) => println!("  numérico:  {e}"),
        },
        Ok(_) => {}
        Err(e) => println!("Parâmetros: {e}"),
    }
    println!();
}

fn usage() -> ! {
    eprintln!("uso: solve (--list | N | arquivo.json ... | --json N)");
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => usage(),
        Some("--list") => {
            for (i, (name, _)) in examples::all().iter().enumerate() {
                println!("{:>2}. {name}", i + 1);
            }
        }
        Some("--json") => {
            let Some(a) = args.get(1) else { usage() };
            match load(a) {
                Ok((_, d)) => println!("{}", d.to_json()),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        Some(_) => {
            for a in &args {
                match load(a) {
                    Ok((name, d)) => run(&name, &d),
                    Err(e) => eprintln!("{e}"),
                }
            }
        }
    }
}
