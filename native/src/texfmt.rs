//! Text and number formatting shared by the result views: pt-BR numbers and a readable
//! Unicode fallback for TeX that is shown until (or instead of) the typeset formula.
use serde_json::Value;

pub fn list(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

pub fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}

pub fn raw(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

/// `formatDisplayNumber` from the previous renderer: pt-BR, at most 3 decimals.
pub fn display_number(value: &Value) -> String {
    match value {
        Value::Null => "—".into(),
        Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), |n| pt_br(n, 3)),
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "—" || trimmed == "None" {
                return "—".into();
            }
            match trimmed.parse::<f64>() {
                Ok(n) if n.is_finite() => pt_br(n, 3),
                _ => decimal_commas(s),
            }
        }
        other => other.to_string(),
    }
}

/// `Number.toLocaleString('pt-BR', {maximumFractionDigits})`.
pub fn pt_br(n: f64, digits: usize) -> String {
    if !n.is_finite() {
        return n.to_string();
    }
    let fixed = format!("{n:.digits$}");
    let fixed = if fixed.contains('.') {
        fixed.trim_end_matches('0').trim_end_matches('.')
    } else {
        &fixed
    };
    let (sign, digits) = fixed.strip_prefix('-').map_or(("", fixed), |d| ("-", d));
    let (int, frac) = digits.split_once('.').unwrap_or((digits, ""));
    let mut grouped = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(ch);
    }
    let sign = if grouped == "0" && frac.is_empty() {
        ""
    } else {
        sign
    };
    if frac.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped},{frac}")
    }
}

/// Fixed decimals with a decimal comma, as the previous memorial tables showed.
pub fn fixed(n: f64, digits: usize) -> String {
    format!("{n:.digits$}").replace('.', ",")
}

/// Decimal points between digits become commas; other text is untouched.
pub fn decimal_commas(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let between = i > 0
                && chars[i - 1].is_ascii_digit()
                && chars.get(i + 1).is_some_and(char::is_ascii_digit);
            if c == '.' && between { ',' } else { c }
        })
        .collect()
}

/// Readable Unicode approximation of TeX, shown until (or instead of) the typeset image.
pub fn tex_to_text(tex: &str) -> String {
    let chars: Vec<char> = tex.chars().collect();
    let mut i = 0;
    convert(&chars, &mut i, false)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn convert(c: &[char], i: &mut usize, group: bool) -> String {
    let mut out = String::new();
    while *i < c.len() {
        let ch = c[*i];
        *i += 1;
        match ch {
            '}' if group => return out,
            '{' => out.push_str(&convert(c, i, true)),
            '$' => {}
            '^' | '_' => {
                let arg = argument(c, i);
                out.push_str(&script(&arg, ch == '^'));
            }
            '\\' => out.push_str(&command(c, i)),
            '&' | '~' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

fn argument(c: &[char], i: &mut usize) -> String {
    while c.get(*i) == Some(&' ') {
        *i += 1;
    }
    let Some(&ch) = c.get(*i) else {
        return String::new();
    };
    *i += 1;
    match ch {
        '{' => convert(c, i, true),
        '\\' => command(c, i),
        _ => ch.to_string(),
    }
}

fn command(c: &[char], i: &mut usize) -> String {
    let start = *i;
    while c.get(*i).is_some_and(char::is_ascii_alphabetic) {
        *i += 1;
    }
    if *i == start {
        let Some(&ch) = c.get(*i) else {
            return String::new();
        };
        *i += 1;
        return match ch {
            ',' | ';' | ':' | ' ' => " ".into(),
            '!' => String::new(),
            '\\' => "; ".into(),
            other => other.to_string(),
        };
    }
    let name: String = c[start..*i].iter().collect();
    match name.as_str() {
        "frac" | "dfrac" | "tfrac" => {
            let numerator = argument(c, i);
            let denominator = argument(c, i);
            format!("{}/{}", wrap(&numerator), wrap(&denominator))
        }
        "sqrt" => format!("√({})", argument(c, i)),
        "text" | "mathrm" | "mathbf" | "mathit" | "textbf" | "operatorname" | "mathcal" => {
            argument(c, i)
        }
        "mathbb" => {
            let arg = argument(c, i);
            match arg.as_str() {
                "R" => "ℝ".into(),
                "C" => "ℂ".into(),
                "Z" => "ℤ".into(),
                "N" => "ℕ".into(),
                "Q" => "ℚ".into(),
                _ => arg,
            }
        }
        "left" | "right" | "bigl" | "bigr" | "big" | "Big" => {
            if c.get(*i) == Some(&'.') {
                *i += 1;
            }
            String::new()
        }
        "displaystyle" => String::new(),
        "quad" | "qquad" => "  ".into(),
        _ => symbol(&name).map_or(name, str::to_owned),
    }
}

fn wrap(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() > 1 && s.contains([' ', '+', '-', '*', '/', '−']) {
        format!("({s})")
    } else {
        s.to_owned()
    }
}

fn script(arg: &str, sup: bool) -> String {
    let mapped: Option<String> = arg
        .trim()
        .chars()
        .map(|c| {
            if sup {
                superscript_char(c)
            } else {
                subscript_char(c)
            }
        })
        .collect();
    match mapped {
        Some(s) if !s.is_empty() => s,
        _ if arg.trim().chars().count() == 1 => {
            format!("{}{}", if sup { '^' } else { '_' }, arg.trim())
        }
        _ => format!("{}({})", if sup { '^' } else { '_' }, arg.trim()),
    }
}

#[rustfmt::skip]
fn superscript_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '⁰', '1' => '¹', '2' => '²', '3' => '³', '4' => '⁴', '5' => '⁵', '6' => '⁶',
        '7' => '⁷', '8' => '⁸', '9' => '⁹', '+' => '⁺', '-' | '−' => '⁻', '=' => '⁼', '(' => '⁽',
        ')' => '⁾', '°' => '°', '′' => '′',
        _ => return None,
    })
}

#[rustfmt::skip]
fn subscript_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '₀', '1' => '₁', '2' => '₂', '3' => '₃', '4' => '₄', '5' => '₅', '6' => '₆',
        '7' => '₇', '8' => '₈', '9' => '₉', '+' => '₊', '-' | '−' => '₋', '=' => '₌', '(' => '₍',
        ')' => '₎',
        _ => return None,
    })
}

#[rustfmt::skip]
fn symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "α", "beta" => "β", "gamma" => "γ", "delta" => "δ", "Delta" => "Δ",
        "epsilon" | "varepsilon" => "ε", "zeta" => "ζ", "eta" => "η", "theta" => "θ",
        "lambda" => "λ", "mu" => "μ", "xi" => "ξ", "pi" => "π", "rho" => "ρ", "sigma" => "σ",
        "Sigma" => "Σ", "tau" => "τ", "phi" | "varphi" => "φ", "Phi" => "Φ", "omega" => "ω",
        "Omega" => "Ω", "infty" => "∞", "to" | "rightarrow" => "→", "leftarrow" => "←",
        "Rightarrow" | "implies" => "⇒", "Leftrightarrow" | "iff" => "⇔", "le" | "leq" => "≤",
        "ge" | "geq" => "≥", "ne" | "neq" => "≠", "approx" => "≈", "pm" => "±", "mp" => "∓",
        "cdot" => "·", "times" => "×", "div" => "÷", "circ" => "°", "angle" => "∠", "sum" => "Σ",
        "prod" => "Π", "partial" => "∂", "dots" | "ldots" | "cdots" => "…", "in" => "∈",
        "cup" => "∪", "cap" => "∩", "forall" => "∀", "exists" => "∃", "prime" => "′",
        "ast" | "star" => "*", "lbrace" => "{", "rbrace" => "}", "langle" => "⟨", "rangle" => "⟩",
        "lim" => "lim", "max" => "max", "min" => "min", "log" => "log", "ln" => "ln",
        "exp" => "exp", "sin" => "sin", "cos" => "cos", "tan" => "tan", "det" => "det",
        "Re" => "Re", "Im" => "Im", "arctan" => "arctan",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_follow_the_previous_pt_br_formatting() {
        assert_eq!(pt_br(1234.5, 3), "1.234,5");
        assert_eq!(pt_br(12345.678, 3), "12.345,678");
        assert_eq!(pt_br(0.12345, 3), "0,123");
        assert_eq!(pt_br(100.0, 3), "100");
        assert_eq!(pt_br(-3.0, 3), "-3");
        assert_eq!(pt_br(-0.0001, 3), "0");
        assert_eq!(display_number(&json!("16.336")), "16,336");
        assert_eq!(display_number(&json!(null)), "—");
        assert_eq!(display_number(&json!("None")), "—");
        assert_eq!(display_number(&json!("Estável")), "Estável");
        assert_eq!(display_number(&json!("0.34 + 1.5j")), "0,34 + 1,5j");
        assert_eq!(display_number(&json!("∞")), "∞");
    }

    #[test]
    fn tex_reads_as_unicode_text() {
        assert_eq!(tex_to_text(r"\frac{K}{3}"), "K/3");
        assert_eq!(tex_to_text(r"2 - \frac{K}{3} > 0"), "2 - K/3 > 0");
        assert_eq!(tex_to_text(r"s^{3} + \omega_n^2"), "s³ + ω_n²");
        assert_eq!(tex_to_text(r"\left(s + 1\right)"), "(s + 1)");
        assert_eq!(
            tex_to_text(r"\omega_{osc} = 1.414 \text{ rad/s}"),
            "ω_(osc) = 1.414 rad/s"
        );
        assert_eq!(tex_to_text(r"2{,}236"), "2,236");
        assert_eq!(tex_to_text(r"\theta = 60^\circ"), "θ = 60°");
        assert_eq!(tex_to_text(r"\sqrt{1 - \xi^2}"), "√(1 - ξ²)");
    }
}
