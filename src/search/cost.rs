use super::{Entry, Execution, Frontier, Phase, Rastro, Step};
use crate::graph::{NodeId, WeightedGraph};
use std::collections::{BinaryHeap, HashMap, HashSet};

pub fn ucs(grafo: &WeightedGraph, inicio: &str, objetivo: &str) -> Execution {
    let mut rastro = Rastro::new();
    let mut fila: BinaryHeap<Entry> = BinaryHeap::new();
    let mut visitados: Vec<NodeId> = Vec::new();
    let mut conjunto: HashSet<NodeId> = HashSet::new();
    let mut g_conhecido: HashMap<NodeId, f64> = HashMap::new();
    let mut expandidos = 0usize;
    let mut seq = 0usize;

    fila.push(Entry::new(0.0, inicio, vec![inicio.to_string()], 0.0, seq));
    seq += 1;
    g_conhecido.insert(inicio.to_string(), 0.0);
    rastro.registrar(Step {
        phase: Phase::Init,
        node: Some(inicio.to_string()),
        message: format!("fila de prioridade com {inicio} (g = 0). O menor custo sai primeiro"),
        frontier: fila.items(),
        ..Default::default()
    });

    while let Some(entrada) = fila.pop() {
        let no = entrada.node.clone();
        let caminho = entrada.path.clone();
        let profundidade = caminho.len() - 1;
        rastro.registrar(Step {
            phase: Phase::Pop,
            node: Some(no.clone()),
            g: entrada.g,
            depth: profundidade,
            message: format!("{no} saiu da fila (menor g = {})", entrada.g),
            frontier: fila.items(),
            visited: visitados.clone(),
            path: caminho.clone(),
            ..Default::default()
        });

        if conjunto.contains(&no) {
            rastro.registrar(Step {
                phase: Phase::Cut,
                node: Some(no.clone()),
                g: entrada.g,
                depth: profundidade,
                message: format!(
                    "{no} já estava fechado (g = {}): entrada antiga descartada",
                    entrada.g
                ),
                frontier: fila.items(),
                visited: visitados.clone(),
                path: caminho,
                ..Default::default()
            });
            continue;
        }

        if no == objetivo {
            visitados.push(no.clone());
            let custo = entrada.g;
            rastro.registrar(Step {
                phase: Phase::Goal,
                node: Some(no.clone()),
                g: custo,
                depth: profundidade,
                message: format!("objetivo {no} alcançado com custo total {custo}"),
                frontier: fila.items(),
                visited: visitados.clone(),
                path: caminho.clone(),
                ..Default::default()
            });
            return Execution {
                path: Some(caminho),
                cost: custo,
                expanded: expandidos,
                trace: rastro.trace,
                iterations: vec![],
                corte: false,
            };
        }

        conjunto.insert(no.clone());
        visitados.push(no.clone());
        expandidos += 1;
        let vizinhos: Vec<(NodeId, f64)> = grafo.vizinhos(&no).to_vec();
        let candidatos: Vec<(NodeId, f64)> = vizinhos
            .iter()
            .filter(|(v, _)| !conjunto.contains(v))
            .cloned()
            .collect();
        rastro.registrar(Step {
            phase: Phase::Expand,
            node: Some(no.clone()),
            g: entrada.g,
            depth: profundidade,
            message: format!("expandindo {no} (g = {})", entrada.g),
            frontier: fila.items(),
            visited: visitados.clone(),
            path: caminho.clone(),
            ..Default::default()
        });

        for (vizinho, peso) in candidatos {
            let g_novo = entrada.g + peso;
            if let Some(g_antigo) = g_conhecido.get(&vizinho)
                && *g_antigo <= g_novo
            {
                continue;
            }
            let g_anterior = g_conhecido.insert(vizinho.clone(), g_novo);
            let mut novo_caminho = caminho.clone();
            novo_caminho.push(vizinho.clone());
            fila.push(Entry::new(
                g_novo,
                &vizinho,
                novo_caminho.clone(),
                peso,
                seq,
            ));
            seq += 1;
            let g_atual = entrada.g;
            let motivo = match g_anterior {
                Some(anterior) => format!("melhor que o custo {anterior} já conhecido"),
                None => "primeira vez que este nó entra na fila".to_string(),
            };
            rastro.registrar(Step {
                phase: Phase::Push,
                node: Some(vizinho.clone()),
                g: g_novo,
                depth: novo_caminho.len() - 1,
                message: format!(
                    "{no} -> {vizinho} entrou na fila: g = {g_atual} + {peso} = {g_novo} ({motivo})"
                ),
                frontier: fila.items(),
                visited: visitados.clone(),
                path: novo_caminho,
                ..Default::default()
            });
        }
    }

    let sem_caminho = grafo.existe(objetivo);
    rastro.registrar(Step {
        phase: Phase::Fail,
        visited: visitados,
        message: if sem_caminho {
            "a fila ficou vazia: nenhum caminho leva ao objetivo".to_string()
        } else {
            "a fila ficou vazia: o objetivo não existe no grafo".to_string()
        },
        ..Default::default()
    });

    Execution {
        path: None,
        cost: 0.0,
        expanded: expandidos,
        trace: rastro.trace,
        iterations: vec![],
        corte: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Graph};
    use crate::search::exemplos::{grafo_classico, nos};

    fn ponderado(edges: Vec<Edge>, nodes: Vec<&str>) -> WeightedGraph {
        let grafo = Graph {
            directed: false,
            nodes: nos(&nodes),
            edges,
        };
        WeightedGraph::from_graph(&grafo).unwrap()
    }

    #[test]
    fn achada_a_solucao_de_custo_minimo() {
        let g = ponderado(
            vec![
                Edge::nova("A", "B", 4.0),
                Edge::nova("A", "C", 2.0),
                Edge::nova("B", "C", 1.0),
                Edge::nova("B", "D", 2.0),
                Edge::nova("C", "D", 5.0),
                Edge::nova("C", "E", 10.0),
                Edge::nova("D", "E", 1.0),
            ],
            vec!["A", "B", "C", "D", "E"],
        );
        let r = ucs(&g, "A", "E");
        assert_eq!(
            r.path,
            Some(vec![
                "A".into(),
                "C".into(),
                "B".into(),
                "D".into(),
                "E".into()
            ])
        );
        assert_eq!(r.cost, 6.0);
    }

    #[test]
    fn fronteira_sempre_ordenada_por_custo() {
        let r = ucs(
            &WeightedGraph::from_graph(&grafo_classico()).unwrap(),
            "A",
            "E",
        );
        for passo in &r.trace {
            let mut copia: Vec<f64> = passo.frontier.iter().map(|i| i.g).collect();
            copia.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let original: Vec<f64> = passo.frontier.iter().map(|i| i.g).collect();
            assert_eq!(original, copia, "passo fora de ordem: {passo:?}");
        }
    }

    #[test]
    fn conjunto_fechado_so_cresce() {
        let r = ucs(
            &WeightedGraph::from_graph(&grafo_classico()).unwrap(),
            "A",
            "E",
        );
        let mut tamanho = 0;
        for passo in &r.trace {
            assert!(passo.visited.len() >= tamanho);
            tamanho = passo.visited.len();
        }
    }

    #[test]
    fn cada_entrada_da_fila_mostra_o_caminho_ate_ela() {
        let r = ucs(
            &WeightedGraph::from_graph(&grafo_classico()).unwrap(),
            "A",
            "E",
        );
        let item = r
            .trace
            .iter()
            .flat_map(|p| p.frontier.iter())
            .find(|i| i.node == "D")
            .unwrap();
        assert_eq!(item.path.first().map(String::as_str), Some("A"));
        assert_eq!(item.path.last().map(String::as_str), Some("D"));
    }

    #[test]
    fn objetivo_inalcancavel_encerra_com_falha() {
        let g = ponderado(vec![Edge::nova("A", "B", 1.0)], vec!["A", "B", "Z"]);
        let r = ucs(&g, "A", "Z");
        assert_eq!(r.path, None);
        assert_eq!(r.trace.last().map(|p| p.phase), Some(Phase::Fail));
    }

    #[test]
    fn custo_zero_e_respeitado() {
        let g = ponderado(
            vec![Edge::nova("A", "B", 0.0), Edge::nova("B", "C", 0.0)],
            vec!["A", "B", "C"],
        );
        let r = ucs(&g, "A", "C");
        assert!(r.path.is_some());
        assert_eq!(r.cost, 0.0);
    }
}
