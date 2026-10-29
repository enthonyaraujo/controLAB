//! Diagrama de blocos: modelo editável (serializado em JSON), validação
//! e exemplos prontos.

pub mod examples;
mod model;
mod validate;

pub use model::{
    ConnectError, Diagram, FORMAT_VERSION, LoadError, Node, NodeId, NodeKind, PortDir, PortRef,
    Pos, Side, Sign, SumInput, Wire, WireId,
};
pub use validate::{BlockGains, DiagramError, ParamError, validate};
