use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

pub type NodeId = String;

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub id: NodeId,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    #[serde(default)]
    pub directed: bool,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErroGrafo {
    pub erro: String,
}

impl ErroGrafo {
    pub fn novo(mensagem: impl Into<String>) -> Self {
        ErroGrafo {
            erro: mensagem.into(),
        }
    }
}

impl fmt::Display for ErroGrafo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.erro)
    }
}

impl std::error::Error for ErroGrafo {}

impl From<ErroGrafo> for String {
    fn from(erro: ErroGrafo) -> Self {
        erro.erro
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NoAux {
    Rotulo(String),
    Completo {
        id: String,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        x: Option<f64>,
        #[serde(default)]
        y: Option<f64>,
    },
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let aux = NoAux::deserialize(deserializer)?;
        let (id, label, x, y) = match aux {
            NoAux::Rotulo(id) => (id, None, None, None),
            NoAux::Completo { id, label, x, y } => (id, label, x, y),
        };
        Ok(Node { id, label, x, y })
    }
}

impl Node {
    pub fn novo(id: impl Into<String>) -> Self {
        let id = id.into();
        Node {
            label: Some(id.clone()),
            id,
            x: None,
            y: None,
        }
    }

    pub fn rotulo(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.id)
    }
}

impl Edge {
    pub fn nova(from: impl Into<String>, to: impl Into<String>, weight: f64) -> Self {
        Edge {
            from: from.into(),
            to: to.into(),
            weight,
        }
    }
}

impl Graph {
    pub fn validar(&self) -> Result<(), ErroGrafo> {
        if self.nodes.is_empty() {
            return Err(ErroGrafo::novo("o grafo precisa ter pelo menos um nó"));
        }

        let mut vistos: HashSet<&str> = HashSet::new();
        for no in &self.nodes {
            let id = no.id.trim();
            if id.is_empty() {
                return Err(ErroGrafo::novo("existe um nó sem identificador"));
            }
            if !vistos.insert(id) {
                return Err(ErroGrafo::novo(format!("nó duplicado: {id}")));
            }
        }

        for (i, aresta) in self.edges.iter().enumerate() {
            if !vistos.contains(aresta.from.as_str()) {
                return Err(ErroGrafo::novo(format!(
                    "aresta #{i} sai do nó inexistente \"{}\"",
                    aresta.from
                )));
            }
            if !vistos.contains(aresta.to.as_str()) {
                return Err(ErroGrafo::novo(format!(
                    "aresta #{i} chega em um nó inexistente: \"{}\"",
                    aresta.to
                )));
            }
            if !aresta.weight.is_finite() || aresta.weight < 0.0 {
                return Err(ErroGrafo::novo(format!(
                    "a aresta {} -> {} precisa de um custo finito e >= 0 (veio {})",
                    aresta.from, aresta.to, aresta.weight
                )));
            }
        }

        Ok(())
    }

    pub fn nos(&self) -> Vec<&Node> {
        self.nodes.iter().collect()
    }
}

pub struct WeightedGraph {
    pub directed: bool,
    adjacencias: HashMap<NodeId, Vec<(NodeId, f64)>>,
    ordem: Vec<NodeId>,
}

impl WeightedGraph {
    pub fn new(params: HashMap<NodeId, Vec<(NodeId, f64)>>) -> Self {
        let ordem: Vec<NodeId> = params
            .iter()
            .flat_map(|(no, vizinhos)| {
                std::iter::once(no.clone()).chain(vizinhos.iter().map(|(v, _)| v.clone()))
            })
            .collect();
        WeightedGraph {
            directed: false,
            adjacencias: params,
            ordem,
        }
    }

    pub fn from_graph(grafo: &Graph) -> Result<Self, ErroGrafo> {
        grafo.validar()?;
        Ok(WeightedGraph {
            directed: grafo.directed,
            adjacencias: Self::construir_adjacencias(grafo),
            ordem: grafo.nodes.iter().map(|no| no.id.clone()).collect(),
        })
    }

    fn construir_adjacencias(grafo: &Graph) -> HashMap<NodeId, Vec<(NodeId, f64)>> {
        let mut adjacencias: HashMap<NodeId, Vec<(NodeId, f64)>> = grafo
            .nodes
            .iter()
            .map(|no| (no.id.clone(), Vec::new()))
            .collect();

        for aresta in &grafo.edges {
            Self::ligar(&mut adjacencias, &aresta.from, &aresta.to, aresta.weight);
            if !grafo.directed {
                Self::ligar(&mut adjacencias, &aresta.to, &aresta.from, aresta.weight);
            }
        }

        adjacencias
    }

    fn ligar(
        adjacencias: &mut HashMap<NodeId, Vec<(NodeId, f64)>>,
        de: &str,
        para: &str,
        peso: f64,
    ) {
        let lista = adjacencias.entry(de.to_string()).or_default();
        if !lista.iter().any(|(v, _)| v == para) {
            lista.push((para.to_string(), peso));
        }
    }

    pub fn nos(&self) -> &[NodeId] {
        &self.ordem
    }

    pub fn existe(&self, no: &str) -> bool {
        self.adjacencias.contains_key(no)
    }

    pub fn vizinhos(&self, no: &str) -> &[(NodeId, f64)] {
        self.adjacencias
            .get(no)
            .map(|lista| lista.as_slice())
            .unwrap_or(&[])
    }

    pub fn custo(&self, de: &str, para: &str) -> Option<f64> {
        self.vizinhos(de)
            .iter()
            .find(|(v, _)| v == para)
            .map(|(_, custo)| *custo)
    }

    pub fn custo_do_caminho(&self, caminho: &[NodeId]) -> f64 {
        caminho
            .windows(2)
            .map(|par| self.custo(&par[0], &par[1]).unwrap_or(f64::INFINITY))
            .sum()
    }

    pub fn bfs(&self, no_inicio: &str, no_objetivo: &str) -> Option<Vec<NodeId>> {
        crate::search::uniform::bfs(self, no_inicio, no_objetivo).path
    }

    pub fn dfs(&self, no_inicio: &str, no_objetivo: &str) -> Option<Vec<NodeId>> {
        crate::search::uniform::dfs(self, no_inicio, no_objetivo, None, "DFS").path
    }

    pub fn ucs(&self, no_inicio: &str, no_objetivo: &str) -> Option<Vec<NodeId>> {
        crate::search::cost::ucs(self, no_inicio, no_objetivo).path
    }

    pub fn dls(
        &self,
        no_inicio: &str,
        no_objetivo: &str,
        max_iterations: usize,
    ) -> Option<Vec<NodeId>> {
        crate::search::depth::dls(self, no_inicio, no_objetivo, Some(max_iterations)).path
    }

    pub fn ids(
        &self,
        no_inicio: &str,
        no_objetivo: &str,
        max_iterations: usize,
    ) -> Option<Vec<NodeId>> {
        crate::search::depth::ids(self, no_inicio, no_objetivo, max_iterations).path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nos(ids: &[&str]) -> Vec<Node> {
        ids.iter().map(|id| Node::novo(*id)).collect()
    }

    #[test]
    fn nos_aceitam_rotulo_simples_ou_objeto() {
        let json =
            r#"{"directed": false, "nodes": ["A", {"id": "B", "label": "bis"}], "edges": []}"#;
        let grafo: Graph = serde_json::from_str(json).unwrap();
        grafo.validar().unwrap();
        assert_eq!(grafo.nodes.len(), 2);
        assert_eq!(grafo.nodes[1].rotulo(), "bis");
    }

    #[test]
    fn detecta_no_duplicado() {
        let grafo = Graph {
            directed: false,
            nodes: nos(&["A", "A"]),
            edges: vec![],
        };
        assert!(grafo.validar().is_err());
    }

    #[test]
    fn detecta_aresta_para_no_inexistente() {
        let grafo = Graph {
            directed: false,
            nodes: nos(&["A"]),
            edges: vec![Edge::nova("A", "Z", 1.0)],
        };
        let erro = grafo.validar().unwrap_err();
        assert!(erro.erro.contains("Z"));
    }

    #[test]
    fn detecta_custo_negativo() {
        let grafo = Graph {
            directed: false,
            nodes: nos(&["A", "B"]),
            edges: vec![Edge::nova("A", "B", -3.0)],
        };
        assert!(grafo.validar().is_err());
    }

    #[test]
    fn grafo_nao_dirigido_fica_simetrico() {
        let grafo = Graph {
            directed: false,
            nodes: nos(&["A", "B"]),
            edges: vec![Edge::nova("A", "B", 4.0)],
        };
        let ponderado = WeightedGraph::from_graph(&grafo).unwrap();
        assert_eq!(ponderado.custo("A", "B"), Some(4.0));
        assert_eq!(ponderado.custo("B", "A"), Some(4.0));
        assert_eq!(ponderado.custo_do_caminho(&["A".into(), "B".into()]), 4.0);
    }

    #[test]
    fn grafo_dirigido_nao_cria_sentido_reverso() {
        let grafo = Graph {
            directed: true,
            nodes: nos(&["A", "B"]),
            edges: vec![Edge::nova("A", "B", 4.0)],
        };
        let ponderado = WeightedGraph::from_graph(&grafo).unwrap();
        assert_eq!(ponderado.custo("A", "B"), Some(4.0));
        assert_eq!(ponderado.custo("B", "A"), None);
    }

    #[test]
    fn aresta_repetida_nao_duplica_na_adjacencia() {
        let grafo = Graph {
            directed: false,
            nodes: nos(&["A", "B"]),
            edges: vec![Edge::nova("A", "B", 4.0), Edge::nova("B", "A", 4.0)],
        };
        let ponderado = WeightedGraph::from_graph(&grafo).unwrap();
        assert_eq!(ponderado.vizinhos("A").len(), 1);
    }
}
