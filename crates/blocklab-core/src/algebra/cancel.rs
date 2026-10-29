//! Cancelamento cooperativo de cálculos longos.
//!
//! A interface roda o `solve` numa thread de trabalho e, se o usuário
//! editar o diagrama no meio do cálculo, pede para parar. Rust não mata
//! threads à força: quem calcula precisa *olhar* uma flag de vez em
//! quando e desistir sozinho.
//!
//! O Mason recebe o [`CancelToken`] explicitamente. O MDC, porém, é
//! chamado de dentro dos operadores `+`, `*`... e não dá para passar um
//! parâmetro extra por eles. Por isso o token também fica numa variável
//! *local da thread* ([`with_cancel`]): o MDC consulta essa variável e, se
//! o cálculo foi cancelado, devolve 1 na hora. Esse "1" deixa a fração
//! sem simplificar (ainda correta, só não canônica), mas não importa: o
//! Mason confere o token logo depois e descarta tudo com `Cancelled`.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Flag compartilhada entre quem pede o cancelamento (a interface) e
/// quem calcula. Clonar compartilha a mesma flag.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pede o cancelamento (pode ser chamado de qualquer thread).
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

thread_local! {
    static CURRENT: RefCell<Option<CancelToken>> = const { RefCell::new(None) };
}

/// Executa `f` com `token` visível para a álgebra desta thread.
/// Restaura o token anterior ao sair, mesmo se `f` entrar em pânico.
pub fn with_cancel<R>(token: &CancelToken, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<CancelToken>);
    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT.with(|c| *c.borrow_mut() = self.0.take());
        }
    }
    let previous = CURRENT.with(|c| c.borrow_mut().replace(token.clone()));
    let _restore = Restore(previous);
    f()
}

/// `true` se o cálculo em andamento nesta thread foi cancelado.
pub(crate) fn cancelled() -> bool {
    CURRENT.with(|c| c.borrow().as_ref().is_some_and(CancelToken::is_cancelled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_visivel_so_dentro_do_escopo() {
        let t = CancelToken::new();
        t.cancel();
        assert!(!cancelled());
        with_cancel(&t, || assert!(cancelled()));
        assert!(!cancelled());
    }
}
