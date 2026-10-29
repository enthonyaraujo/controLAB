//! A saída de texto da formatação, lida de volta pelo parser, precisa dar
//! exatamente a mesma função — em qualquer modo e nos dois estilos.
//! Assim, o que a interface mostra (e o usuário copia) nunca mente.

mod common;

use blocklab_core::format::{Mode, to_ascii, to_text};
use blocklab_core::parser::parse_rational;
use common::rational;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn texto_volta_igual(f in rational()) {
        for mode in [Mode::Symbolic, Mode::Numeric] {
            for text in [to_text(&f, mode), to_ascii(&f, mode)] {
                let back = parse_rational(&text).map_err(|e| e.render(&text));
                prop_assert_eq!(back, Ok(f.clone()), "texto: {}", text);
            }
        }
    }
}
