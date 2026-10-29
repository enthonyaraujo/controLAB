//! Modelo do diagrama de blocos: nós, portas e fios. É o que a interface
//! edita e o que vai para o arquivo `.json`.
//!
//! O modelo não sabe nada de matemática: um bloco guarda a expressão como
//! texto. A conversão para ganhos e grafo de fluxo de sinal fica em
//! `validate.rs` e `sfg/build.rs`.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Identificador estável de nó (não muda ao apagar outros nós).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u64);

/// Identificador estável de fio.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WireId(pub u64);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Posição no canvas (coordenadas do "mundo", antes de zoom/pan).
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Pos { x, y }
    }
}

/// Sinal de uma entrada do somador.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Sign {
    #[serde(rename = "+")]
    Plus,
    #[serde(rename = "-")]
    Minus,
}

impl Sign {
    /// Alterna + ↔ − (clique no sinal do somador).
    pub fn toggled(self) -> Self {
        match self {
            Sign::Plus => Sign::Minus,
            Sign::Minus => Sign::Plus,
        }
    }
}

/// Lado do nó por onde a seta entra (só afeta o desenho).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Top,
    Bottom,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SumInput {
    pub sign: Sign,
    pub side: Side,
}

/// Tipo do nó e seus dados próprios.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeKind {
    /// Sinal de entrada X(s). Uma porta de saída.
    Input { name: String },
    /// Sinal de saída Y(s). Uma porta de entrada.
    Output { name: String },
    /// Bloco com função de transferência. Uma entrada, uma saída.
    Block { expr: String },
    /// Somador: saída = Σ (±entrada). N entradas, uma saída.
    Sum { inputs: Vec<SumInput> },
    /// Ponto de ramificação: copia o sinal de entrada para N saídas.
    Branch { outputs: u8 },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub pos: Pos,
    /// Desenhado da direita para a esquerda (blocos de realimentação).
    #[serde(default)]
    pub flipped: bool,
    #[serde(flatten)]
    pub kind: NodeKind,
}

impl Node {
    pub fn n_inputs(&self) -> usize {
        match &self.kind {
            NodeKind::Input { .. } => 0,
            NodeKind::Sum { inputs } => inputs.len(),
            _ => 1,
        }
    }

    pub fn n_outputs(&self) -> usize {
        match &self.kind {
            NodeKind::Output { .. } => 0,
            NodeKind::Branch { outputs } => *outputs as usize,
            _ => 1,
        }
    }

    /// Descrição curta para mensagens: "bloco 'G1' (#7)".
    pub fn describe(&self) -> String {
        let what = match &self.kind {
            NodeKind::Input { name } => format!("entrada {name}"),
            NodeKind::Output { name } => format!("saída {name}"),
            NodeKind::Block { expr } => format!("bloco '{expr}'"),
            NodeKind::Sum { .. } => "somador".to_string(),
            NodeKind::Branch { .. } => "ramificação".to_string(),
        };
        format!("{what} ({})", self.id)
    }
}

/// Entrada (`In`) ou saída (`Out`) de um nó.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PortDir {
    In,
    Out,
}

impl fmt::Display for PortDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PortDir::In => "entrada",
            PortDir::Out => "saída",
        })
    }
}

/// Uma porta: nó + índice (entradas e saídas numeradas separadamente).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct PortRef {
    pub node: NodeId,
    pub port: u8,
}

impl PortRef {
    pub fn new(node: NodeId, port: u8) -> Self {
        PortRef { node, port }
    }
}

/// Fio: sempre de uma porta de saída (`from`) para uma de entrada (`to`).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Wire {
    pub id: WireId,
    pub from: PortRef,
    pub to: PortRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConnectError {
    #[error("nó {0} não existe")]
    NoSuchNode(NodeId),
    #[error("o nó {node} não tem a porta de {dir} {port}")]
    NoSuchPort {
        node: NodeId,
        port: u8,
        dir: PortDir,
    },
    #[error("a porta de entrada {port} do nó {node} já tem um fio")]
    InputTaken { node: NodeId, port: u8 },
}

/// Versão do formato de arquivo gravada por este programa. Sobe quando o
/// JSON mudar de um jeito que versões antigas não saibam ler.
pub const FORMAT_VERSION: u32 = 1;

fn version_one() -> u32 {
    1
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("JSON inválido: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "o arquivo é da versão {found} do formato, e este BlockLab só lê até a \
         versão {supported}; atualize o programa"
    )]
    NewerVersion { found: u32, supported: u32 },
}

/// O diagrama completo.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Diagram {
    /// Versão do formato (arquivos sem o campo são da versão 1).
    #[serde(default = "version_one")]
    pub format_version: u32,
    pub nodes: Vec<Node>,
    pub wires: Vec<Wire>,
    /// Valores dos parâmetros para o modo numérico, como texto exato
    /// ("0.1", "1/3", "10"). Ver [`Diagram::numeric_params`].
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    /// Próximo id livre (compartilhado entre nós e fios).
    #[serde(default)]
    next_id: u64,
}

impl Default for Diagram {
    fn default() -> Self {
        Diagram {
            format_version: FORMAT_VERSION,
            nodes: Vec::new(),
            wires: Vec::new(),
            params: BTreeMap::new(),
            next_id: 0,
        }
    }
}

impl Diagram {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lê de JSON e garante que `next_id` não colida com ids existentes
    /// (arquivos escritos à mão podem omitir ou errar esse campo).
    pub fn from_json(text: &str) -> Result<Self, LoadError> {
        let mut d: Diagram = serde_json::from_str(text)?;
        if d.format_version > FORMAT_VERSION {
            return Err(LoadError::NewerVersion {
                found: d.format_version,
                supported: FORMAT_VERSION,
            });
        }
        // Ao gravar de novo, sai na versão atual.
        d.format_version = FORMAT_VERSION;
        let max_id = d
            .nodes
            .iter()
            .map(|n| n.id.0)
            .chain(d.wires.iter().map(|w| w.id.0))
            .max();
        if let Some(m) = max_id {
            d.next_id = d.next_id.max(m + 1);
        }
        Ok(d)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("o modelo sempre serializa")
    }

    fn fresh_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_node(&mut self, kind: NodeKind, pos: Pos) -> NodeId {
        let id = NodeId(self.fresh_id());
        self.nodes.push(Node {
            id,
            pos,
            flipped: false,
            kind,
        });
        id
    }

    pub fn add_input(&mut self, name: &str, pos: Pos) -> NodeId {
        self.add_node(NodeKind::Input { name: name.into() }, pos)
    }

    pub fn add_output(&mut self, name: &str, pos: Pos) -> NodeId {
        self.add_node(NodeKind::Output { name: name.into() }, pos)
    }

    pub fn add_block(&mut self, expr: &str, pos: Pos) -> NodeId {
        self.add_node(NodeKind::Block { expr: expr.into() }, pos)
    }

    /// Somador com as entradas dadas (sinal e lado de entrada).
    pub fn add_sum(&mut self, inputs: &[(Sign, Side)], pos: Pos) -> NodeId {
        let inputs = inputs
            .iter()
            .map(|&(sign, side)| SumInput { sign, side })
            .collect();
        self.add_node(NodeKind::Sum { inputs }, pos)
    }

    pub fn add_branch(&mut self, outputs: u8, pos: Pos) -> NodeId {
        self.add_node(NodeKind::Branch { outputs }, pos)
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Fio que chega numa porta de entrada, se houver.
    pub fn wire_into(&self, port: PortRef) -> Option<&Wire> {
        self.wires.iter().find(|w| w.to == port)
    }

    /// Liga uma porta de saída a uma de entrada. Uma saída pode alimentar
    /// vários fios (ramificação implícita); uma entrada aceita só um.
    pub fn connect(&mut self, from: PortRef, to: PortRef) -> Result<WireId, ConnectError> {
        self.check_port(from, PortDir::Out)?;
        self.check_port(to, PortDir::In)?;
        if self.wire_into(to).is_some() {
            return Err(ConnectError::InputTaken {
                node: to.node,
                port: to.port,
            });
        }
        let id = WireId(self.fresh_id());
        self.wires.push(Wire { id, from, to });
        Ok(id)
    }

    /// Atalho: saída 0 de `a` → entrada `port` de `b`.
    pub fn link(&mut self, a: NodeId, b: NodeId, port: u8) -> Result<WireId, ConnectError> {
        self.connect(PortRef::new(a, 0), PortRef::new(b, port))
    }

    fn check_port(&self, p: PortRef, dir: PortDir) -> Result<(), ConnectError> {
        let node = self.node(p.node).ok_or(ConnectError::NoSuchNode(p.node))?;
        let count = match dir {
            PortDir::In => node.n_inputs(),
            PortDir::Out => node.n_outputs(),
        };
        if (p.port as usize) < count {
            Ok(())
        } else {
            Err(ConnectError::NoSuchPort {
                node: p.node,
                port: p.port,
                dir,
            })
        }
    }

    /// Remove um nó e todos os fios ligados a ele.
    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.retain(|n| n.id != id);
        self.wires.retain(|w| w.from.node != id && w.to.node != id);
    }

    pub fn remove_wire(&mut self, id: WireId) {
        self.wires.retain(|w| w.id != id);
    }
}
