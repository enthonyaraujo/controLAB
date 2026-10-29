//! Ida e volta: qualquer função racional, impressa como texto e lida de
//! novo pelo parser, precisa voltar idêntica. Testa o parser contra
//! centenas de expressões com frações, sinais, potências e símbolos.

mod common;

use blocklab_core::parser::parse_rational;
use common::rational;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn imprimir_e_ler_de_novo(f in rational()) {
        let text = f.to_string();
        let back = parse_rational(&text).map_err(|e| e.render(&text));
        prop_assert_eq!(back, Ok(f));
    }

    #[test]
    fn parser_nunca_entra_em_panico(src in "[-+*/^() .0-9sGHa1_$]{0,24}") {
        // Lixo aleatório: pode dar erro, mas nunca pânico, e o trecho do
        // erro precisa estar dentro do texto (ou logo após o fim).
        if let Err(e) = parse_rational(&src) {
            prop_assert!(e.span.start <= src.chars().count());
            prop_assert!(e.span.start < e.span.end);
        }
    }
}
