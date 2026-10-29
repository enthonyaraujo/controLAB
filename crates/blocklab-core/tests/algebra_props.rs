//! Testes de propriedade: em vez de exemplos escolhidos a dedo, o
//! proptest sorteia centenas de funções racionais e verifica leis que
//! precisam valer para *qualquer* entrada. Se uma falhar, ele "encolhe"
//! o caso até o menor contraexemplo.
//!
//! Como a forma canônica é única, "igualdade matemática" vira `==`.

mod common;

use blocklab_core::algebra::{Params, Symbol, gcd};
use common::{nonzero_poly, nonzero_rational, rational};
use num_rational::BigRational;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        // Guarda contra regressão de desempenho: um caso que demore mais
        // que isso (em debug) falha em vez de travar a suíte.
        timeout: 10_000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn soma_comutativa(a in rational(), b in rational()) {
        prop_assert_eq!(&a + &b, &b + &a);
    }

    #[test]
    fn produto_comutativo(a in rational(), b in rational()) {
        prop_assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn soma_associativa(a in rational(), b in rational(), c in rational()) {
        prop_assert_eq!(&(&a + &b) + &c, &a + &(&b + &c));
    }

    #[test]
    fn produto_associativo(a in rational(), b in rational(), c in rational()) {
        prop_assert_eq!(&(&a * &b) * &c, &a * &(&b * &c));
    }

    #[test]
    fn distributiva(a in rational(), b in rational(), c in rational()) {
        prop_assert_eq!(&a * &(&b + &c), &(&a * &b) + &(&a * &c));
    }

    #[test]
    fn fracao_vezes_inversa_e_um(a in nonzero_rational(), b in nonzero_rational()) {
        let ab = &a / &b;
        let ba = &b / &a;
        prop_assert!((&ab * &ba).is_one());
    }

    #[test]
    fn subtrair_de_si_mesmo_da_zero(a in rational()) {
        prop_assert!((&a - &a).is_zero());
    }

    #[test]
    fn mdc_divide_os_dois(p in nonzero_poly(), q in nonzero_poly(), r in nonzero_poly()) {
        // Constrói um fator comum garantido: p·r e q·r.
        let (pr, qr) = (&p * &r, &q * &r);
        let g = gcd(&pr, &qr);
        prop_assert!(pr.exact_div(&g).is_some());
        prop_assert!(qr.exact_div(&g).is_some());
        // ... e o fator comum r divide o MDC.
        prop_assert!(g.exact_div(&r).is_some());
    }

    #[test]
    fn substituir_preserva_produto(a in rational(), b in rational(), x in -4i64..=4, y in -4i64..=4) {
        let mut params = Params::new();
        params.insert(Symbol::new("a"), BigRational::from_integer(x.into()));
        params.insert(Symbol::new("G1"), BigRational::from_integer(y.into()));
        let (sa, sb) = (a.substitute(&params), b.substitute(&params));
        // Só vale quando nenhum denominador zera com esses valores.
        prop_assume!(sa.is_ok() && sb.is_ok());
        let lhs = (&a * &b).substitute(&params).unwrap();
        prop_assert_eq!(lhs, &sa.unwrap() * &sb.unwrap());
    }
}
