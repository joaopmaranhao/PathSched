pub mod cost;
pub mod depth;
pub mod uniform;

use crate::graph::{ErroGrafo, Graph, NodeId, WeightedGraph};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

pub const MAX_ITERACOES_PADRAO: usize = 50;

pub const BFS: &str = "bfs";
pub const DFS: &str = "dfs";
pub const UCS: &str = "ucs";
pub const IDS: &str = "ids";
pub const DLS: &str = "dls";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Init,
    Push,
    Pop,
    Expand,
    Visit,
    Goal,
    Backtrack,
    Cut,
    Fail,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrontierItem {
    pub node: NodeId,
    pub g: f64,
    pub weight: f64,
    pub path: Vec<NodeId>,
    #[serde(default)]
    pub seq: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    #[serde(default)]
    pub index: usize,
    #[serde(default)]
    pub phase: Phase,
    #[serde(default)]
    pub node: Option<NodeId>,
    #[serde(default)]
    pub g: f64,
    #[serde(default)]
    pub depth: usize,
    #[serde(default)]
    pub iteration: usize,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub frontier: Vec<FrontierItem>,
    #[serde(default)]
    pub visited: Vec<NodeId>,
    #[serde(default)]
    pub call_stack: Vec<NodeId>,
    #[serde(default)]
    pub path: Vec<NodeId>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Iteration {
    pub limit: usize,
    pub steps: usize,
    pub found: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResult {
    pub algorithm: String,
    pub start: NodeId,
    pub goal: NodeId,
    pub found: bool,
    pub path: Vec<NodeId>,
    pub total_cost: f64,
    pub expanded: usize,
    pub steps: usize,
    pub exhausted: bool,
    pub iterations: Vec<Iteration>,
    pub trace: Vec<Step>,
}

#[derive(Debug, Clone, Default)]
pub struct Execution {
    pub path: Option<Vec<NodeId>>,
    pub cost: f64,
    pub expanded: usize,
    pub trace: Vec<Step>,
    pub iterations: Vec<Iteration>,
    pub corte: bool,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub g: f64,
    pub node: NodeId,
    pub path: Vec<NodeId>,
    pub weight: f64,
    pub seq: usize,
}

impl Entry {
    pub fn new(g: f64, node: &str, path: Vec<NodeId>, weight: f64, seq: usize) -> Self {
        Entry {
            g,
            node: node.to_string(),
            path,
            weight,
            seq,
        }
    }

    pub fn frontier_item(&self) -> FrontierItem {
        FrontierItem {
            node: self.node.clone(),
            g: self.g,
            weight: self.weight,
            path: self.path.clone(),
            seq: self.seq,
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.g == other.g && self.seq == other.seq
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        // inverte a comparação de custo (para virar min-heap)
        other
            .g
            .partial_cmp(&self.g)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

pub trait Frontier {
    fn items(&self) -> Vec<FrontierItem>;
    fn tamanho(&self) -> usize {
        self.items().len()
    }
}

impl Frontier for BinaryHeap<Entry> {
    fn items(&self) -> Vec<FrontierItem> {
        let mut itens: Vec<FrontierItem> = self.iter().map(Entry::frontier_item).collect();
        itens.sort_by(|a, b| {
            a.g.partial_cmp(&b.g)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.seq.cmp(&b.seq))
        });
        itens
    }
}

impl Frontier for VecDeque<Entry> {
    fn items(&self) -> Vec<FrontierItem> {
        self.iter().map(Entry::frontier_item).collect()
    }
}

impl Frontier for Vec<Entry> {
    fn items(&self) -> Vec<FrontierItem> {
        self.iter().map(Entry::frontier_item).collect()
    }
}

#[derive(Default)]
pub struct Rastro {
    pub trace: Vec<Step>,
}

impl Rastro {
    pub fn new() -> Self {
        Rastro::default()
    }

    pub fn registrar(&mut self, mut passo: Step) {
        passo.index = self.trace.len();
        self.trace.push(passo);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Requisicao {
    pub graph: Graph,
    pub algorithm: String,
    pub start: String,
    pub goal: String,
    #[serde(default = "max_iteracoes_padrao")]
    pub max_iterations: usize,
    #[serde(default)]
    pub limit: Option<usize>,
}

fn max_iteracoes_padrao() -> usize {
    MAX_ITERACOES_PADRAO
}

pub fn executar(requisicao: &Requisicao) -> Result<SearchResult, ErroGrafo> {
    let algoritmo = requisicao.algorithm.to_lowercase();
    let ponderado = WeightedGraph::from_graph(&requisicao.graph)?;

    if !ponderado.existe(&requisicao.start) {
        return Err(ErroGrafo::novo(format!(
            "nó de início \"{}\" não existe no grafo",
            requisicao.start
        )));
    }
    if !ponderado.existe(&requisicao.goal) {
        return Err(ErroGrafo::novo(format!(
            "nó objetivo \"{}\" não existe no grafo",
            requisicao.goal
        )));
    }

    let execucao = match algoritmo.as_str() {
        BFS => uniform::bfs(&ponderado, &requisicao.start, &requisicao.goal),
        DFS => uniform::dfs(&ponderado, &requisicao.start, &requisicao.goal, None, "DFS"),
        UCS => cost::ucs(&ponderado, &requisicao.start, &requisicao.goal),
        DLS => depth::dls(
            &ponderado,
            &requisicao.start,
            &requisicao.goal,
            requisicao.limit,
        ),
        IDS => depth::ids(
            &ponderado,
            &requisicao.start,
            &requisicao.goal,
            requisicao.max_iterations,
        ),
        outro => {
            return Err(ErroGrafo::novo(format!(
                "algoritmo desconhecido: \"{outro}\" (use bfs, dfs, ucs, ids ou dls)"
            )));
        }
    };

    let achou = execucao.path.is_some();
    let exhausted = algoritmo == IDS && !achou && !execucao.corte;

    Ok(SearchResult {
        algorithm: algoritmo,
        start: requisicao.start.clone(),
        goal: requisicao.goal.clone(),
        found: achou,
        path: execucao.path.unwrap_or_default(),
        total_cost: execucao.cost,
        expanded: execucao.expanded,
        steps: execucao.trace.len(),
        exhausted,
        iterations: execucao.iterations,
        trace: execucao.trace,
    })
}

pub fn algoritmos_disponiveis() -> Vec<(&'static str, &'static str)> {
    vec![
        (UCS, "UCS — Busca de Custo Uniforme (fila de prioridade)"),
        (
            IDS,
            "IDS — Busca Iterativa em Profundidade (limites crescentes)",
        ),
        (DLS, "DLS — Busca em Profundidade Limitada (limite fixo)"),
        (BFS, "BFS — Busca em Largura (não informada, ignora custos)"),
        (
            DFS,
            "DFS — Busca em Profundidade (não informada, ignora custos)",
        ),
    ]
}

#[cfg(test)]
pub(crate) mod exemplos {
    use crate::graph::{Edge, Graph, Node};

    pub fn nos(ids: &[&str]) -> Vec<Node> {
        ids.iter().map(|id| Node::novo(*id)).collect()
    }

    pub fn grafo_classico() -> Graph {
        Graph {
            directed: false,
            nodes: nos(&["A", "B", "C", "D", "E"]),
            edges: vec![
                Edge::nova("A", "B", 4.0),
                Edge::nova("A", "C", 2.0),
                Edge::nova("B", "C", 1.0),
                Edge::nova("B", "D", 2.0),
                Edge::nova("C", "D", 5.0),
                Edge::nova("C", "E", 10.0),
                Edge::nova("D", "E", 1.0),
            ],
        }
    }

    pub fn grafo_custo_vs_saltos() -> Graph {
        Graph {
            directed: false,
            nodes: nos(&["S", "X", "Y", "Z", "T"]),
            edges: vec![
                Edge::nova("S", "X", 1.0),
                Edge::nova("X", "T", 99.0),
                Edge::nova("S", "Y", 1.0),
                Edge::nova("Y", "Z", 1.0),
                Edge::nova("Z", "T", 1.0),
            ],
        }
    }

    pub fn grafo_linhado(n: usize) -> Graph {
        let ids: Vec<String> = (0..n).map(|i| format!("N{i}")).collect();
        let referencias: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let arestas = (1..n)
            .map(|i| Edge::nova(referencias[i - 1], referencias[i], 1.0))
            .collect();
        Graph {
            directed: false,
            nodes: nos(&referencias),
            edges: arestas,
        }
    }

    pub fn grafo_sem_saida() -> Graph {
        Graph {
            directed: false,
            nodes: nos(&["A", "B", "C", "Z"]),
            edges: vec![Edge::nova("A", "B", 1.0), Edge::nova("B", "C", 1.0)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::exemplos::*;
    use super::*;

    fn rodar(algoritmo: &str, grafo: &Graph, inicio: &str, objetivo: &str) -> SearchResult {
        executar(&Requisicao {
            graph: grafo.clone(),
            algorithm: algoritmo.to_string(),
            start: inicio.to_string(),
            goal: objetivo.to_string(),
            max_iterations: MAX_ITERACOES_PADRAO,
            limit: None,
        })
        .expect("busca deveria rodar")
    }

    #[test]
    fn ucs_encontra_o_custo_minimo() {
        let r = rodar(UCS, &grafo_classico(), "A", "E");
        assert!(r.found);
        assert_eq!(r.path, vec!["A", "C", "B", "D", "E"]);
        assert_eq!(r.total_cost, 6.0);
    }

    #[test]
    fn bfs_ignora_custo_e_melhor_saltos() {
        let r = rodar(BFS, &grafo_classico(), "A", "E");
        assert!(r.found);
        assert_eq!(r.path, vec!["A", "C", "E"], "2 saltos é o mínimo");
        assert_eq!(r.total_cost, 12.0);
        assert!(
            r.total_cost > rodar(UCS, &grafo_classico(), "A", "E").total_cost,
            "o BFS paga mais caro mesmo com menos saltos"
        );
    }

    #[test]
    fn bfs_pode_ser_mais_caro_que_ucs() {
        let grafo = grafo_custo_vs_saltos();
        let bfs = rodar(BFS, &grafo, "S", "T");
        let ucs = rodar(UCS, &grafo, "S", "T");
        assert_eq!(bfs.path, vec!["S", "X", "T"]);
        assert_eq!(bfs.total_cost, 100.0);
        assert_eq!(ucs.path, vec!["S", "Y", "Z", "T"]);
        assert_eq!(ucs.total_cost, 3.0);
    }

    #[test]
    fn dfs_encontra_um_caminho_valido() {
        let r = rodar(DFS, &grafo_classico(), "A", "E");
        assert!(r.found);
        assert_eq!(r.path.first().map(String::as_str), Some("A"));
        assert_eq!(r.path.last().map(String::as_str), Some("E"));
    }

    #[test]
    fn dls_com_limite_curto_nao_encontra() {
        let mut req = Requisicao {
            graph: grafo_classico(),
            algorithm: DLS.to_string(),
            start: "A".to_string(),
            goal: "E".to_string(),
            max_iterations: MAX_ITERACOES_PADRAO,
            limit: Some(1),
        };
        let r = executar(&req).unwrap();
        assert!(!r.found, "com limite 1 nenhum nó a 2 saltos é alcançado");

        req.limit = Some(2);
        let r = executar(&req).unwrap();
        assert!(r.found);
        assert_eq!(
            r.path,
            vec!["A", "C", "E"],
            "primeiro caminho em profundidade"
        );
        assert_eq!(r.total_cost, 12.0);
    }

    #[test]
    fn ids_cresce_o_limite_ate_encontrar() {
        let r = rodar(IDS, &grafo_classico(), "A", "E");
        assert!(r.found);
        assert_eq!(r.path, vec!["A", "C", "E"]);
        assert_eq!(r.total_cost, 12.0);
        assert!(r.iterations.len() > 1, "esperava mais de uma iteração");
        let limites: Vec<usize> = r.iterations.iter().map(|i| i.limit).collect();
        assert_eq!(limites, vec![0, 1, 2]);
        assert!(!r.exhausted);
    }

    #[test]
    fn ids_esgota_o_espaco_sem_encontrar() {
        let r = rodar(IDS, &grafo_sem_saida(), "A", "Z");
        assert!(!r.found);
        assert!(r.exhausted);
        // 0 e 1 batem no limite; com 2 a busca percorre A-B-C inteiro e desiste
        assert_eq!(r.iterations.len(), 3);
        assert_eq!(r.iterations[2].limit, 2);
    }

    #[test]
    fn objetivo_inalcancavel_nao_e_encontrado() {
        for algoritmo in [BFS, DFS, UCS, IDS] {
            let r = rodar(algoritmo, &grafo_sem_saida(), "A", "Z");
            assert!(!r.found, "{algoritmo} não deveria encontrar Z");
            assert!(r.path.is_empty());
        }
    }

    #[test]
    fn inicio_igual_a_objetivo_e_solucao_imediata() {
        for algoritmo in [BFS, DFS, UCS, IDS, DLS] {
            let r = rodar(algoritmo, &grafo_classico(), "A", "A");
            assert!(r.found, "{algoritmo} deveria resolver A -> A");
            assert_eq!(r.path, vec!["A"]);
            assert_eq!(r.total_cost, 0.0);
        }
    }

    #[test]
    fn erro_para_no_inexistente() {
        let erro = executar(&Requisicao {
            graph: grafo_classico(),
            algorithm: UCS.to_string(),
            start: "Z".to_string(),
            goal: "E".to_string(),
            max_iterations: MAX_ITERACOES_PADRAO,
            limit: None,
        })
        .unwrap_err();
        assert!(erro.erro.contains("Z"));
    }

    #[test]
    fn erro_para_algoritmo_desconhecido() {
        let erro = executar(&Requisicao {
            graph: grafo_classico(),
            algorithm: "astar".to_string(),
            start: "A".to_string(),
            goal: "E".to_string(),
            max_iterations: MAX_ITERACOES_PADRAO,
            limit: None,
        })
        .unwrap_err();
        assert!(erro.erro.contains("astar"));
    }

    #[test]
    fn rastro_comeca_terminando_com_indice_sequencial() {
        let r = rodar(UCS, &grafo_classico(), "A", "E");
        assert!(!r.trace.is_empty());
        for (i, passo) in r.trace.iter().enumerate() {
            assert_eq!(passo.index, i);
        }
        assert_eq!(r.trace.first().map(|p| p.phase), Some(Phase::Init));
        assert_eq!(r.trace.last().map(|p| p.phase), Some(Phase::Goal));
    }

    #[test]
    fn no_ucs_o_conjunto_fechado_so_cresce() {
        let r = rodar(UCS, &grafo_classico(), "A", "E");
        let mut anterior = 0;
        for passo in &r.trace {
            assert!(
                passo.visited.len() >= anterior,
                "visitados não pode diminuir: {passo:?}"
            );
            anterior = passo.visited.len();
        }
    }

    #[test]
    fn no_ucs_a_fronteira_esta_ordenada_por_custo() {
        let r = rodar(UCS, &grafo_classico(), "A", "E");
        for passo in &r.trace {
            let custos: Vec<f64> = passo.frontier.iter().map(|i| i.g).collect();
            let mut ordenada = custos.clone();
            ordenada.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(custos, ordenada, "fronteira fora de ordem em {passo:?}");
        }
    }

    #[test]
    fn cada_iteracao_do_ids_tem_um_limite_crescente() {
        let r = rodar(IDS, &grafo_linhado(6), "N0", "N5");
        assert!(r.found);
        assert_eq!(r.path.len(), 6);
        for (i, iteracao) in r.iterations.iter().enumerate() {
            assert_eq!(iteracao.limit, i);
            assert!(iteracao.steps > 0);
        }
        let ultima = r.iterations.len() - 1;
        assert!(r.iterations[..ultima].iter().all(|i| !i.found));
        assert!(r.iterations[ultima].found);
        assert!(!r.exhausted);
    }

    #[test]
    fn o_caminho_final_sempre_conecta_inicio_e_objetivo() {
        for algoritmo in [BFS, DFS, UCS, IDS, DLS] {
            let r = rodar(algoritmo, &grafo_classico(), "A", "E");
            let caminho = &r.path;
            assert_eq!(caminho.first().map(String::as_str), Some("A"));
            assert_eq!(caminho.last().map(String::as_str), Some("E"));
            for par in caminho.windows(2) {
                let existe = grafo_classico().edges.iter().any(|a| {
                    (a.from == par[0] && a.to == par[1]) || (a.from == par[1] && a.to == par[0])
                });
                assert!(existe, "aresta inexistente entre {par:?}");
            }
        }
    }

    #[test]
    fn resultado_serializa_para_json_com_rastro() {
        let r = rodar(UCS, &grafo_classico(), "A", "E");
        let json = serde_json::to_string(&r).unwrap();
        let de_novo: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(de_novo.path, r.path);
        assert_eq!(de_novo.trace.len(), r.trace.len());
        assert!(json.contains("\"phase\""));
    }
}
