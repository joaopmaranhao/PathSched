use super::{Entry, Execution, Frontier, Phase, Rastro, Step};
use crate::graph::{NodeId, WeightedGraph};
use std::collections::{HashSet, VecDeque};

pub fn bfs(grafo: &WeightedGraph, inicio: &str, objetivo: &str) -> Execution {
    let mut rastro = Rastro::new();
    let mut fila: VecDeque<Entry> = VecDeque::new();
    let mut visitados: Vec<NodeId> = Vec::new();
    let mut conjunto: HashSet<NodeId> = HashSet::new();
    let mut g_conhecido: Vec<(NodeId, f64)> = Vec::new();
    let mut expandidos = 0usize;
    let mut seq = 0usize;

    if inicio == objetivo {
        let caminho = vec![inicio.to_string()];
        rastro.registrar(Step {
            phase: Phase::Goal,
            node: Some(inicio.to_string()),
            message: format!("{inicio} já é o objetivo"),
            path: caminho.clone(),
            ..Default::default()
        });
        return Execution {
            path: Some(caminho),
            cost: 0.0,
            expanded: 0,
            trace: rastro.trace,
            iterations: vec![],
            corte: false,
        };
    }

    fila.push_back(Entry::new(0.0, inicio, vec![inicio.to_string()], 0.0, seq));
    seq += 1;
    rastro.registrar(Step {
        phase: Phase::Init,
        node: Some(inicio.to_string()),
        message: format!("BFS não usa os custos: cada aresta vale 1 salto. Fila inicial: {inicio}"),
        frontier: fila.items(),
        ..Default::default()
    });

    while let Some(entrada) = fila.pop_front() {
        let no = entrada.node.clone();
        let caminho = entrada.path.clone();
        let profundidade = caminho.len() - 1;
        rastro.registrar(Step {
            phase: Phase::Pop,
            node: Some(no.clone()),
            g: entrada.g,
            depth: profundidade,
            message: format!("saiu da frente da fila: {no} (saltos = {})", entrada.g),
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
                message: format!("{no} já tinha sido expandido: entrada duplicada descartada"),
                frontier: fila.items(),
                visited: visitados.clone(),
                path: caminho,
                ..Default::default()
            });
            continue;
        }

        if no == objetivo {
            visitados.push(no.clone());
            let custo = grafo.custo_do_caminho(&caminho);
            rastro.registrar(Step {
                phase: Phase::Goal,
                node: Some(no.clone()),
                g: custo,
                depth: profundidade,
                message: format!(
                    "objetivo {no} alcançado em {profundidade} salto(s) com custo {custo}"
                ),
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
        let candidatos = vizinhos
            .iter()
            .filter(|(v, _)| !conjunto.contains(v))
            .count();
        rastro.registrar(Step {
            phase: Phase::Expand,
            node: Some(no.clone()),
            g: entrada.g,
            depth: profundidade,
            message: format!("expandindo {no} ({candidatos} vizinho(s) ainda não visitados)"),
            frontier: fila.items(),
            visited: visitados.clone(),
            path: caminho.clone(),
            ..Default::default()
        });

        for (vizinho, peso) in vizinhos {
            if conjunto.contains(&vizinho) {
                continue;
            }
            let g_novo = entrada.g + 1.0;
            if let Some((_, g_antigo)) = g_conhecido.iter().find(|(v, _)| *v == vizinho)
                && *g_antigo <= g_novo
            {
                continue;
            }
            g_conhecido.retain(|(v, _)| *v != vizinho);
            g_conhecido.push((vizinho.clone(), g_novo));

            let mut novo_caminho = caminho.clone();
            novo_caminho.push(vizinho.clone());
            fila.push_back(Entry::new(
                g_novo,
                &vizinho,
                novo_caminho.clone(),
                peso,
                seq,
            ));
            seq += 1;
            rastro.registrar(Step {
                phase: Phase::Push,
                node: Some(vizinho.clone()),
                g: g_novo,
                depth: novo_caminho.len() - 1,
                message: format!("{no} -> {vizinho} entrou no fim da fila (saltos = {g_novo})"),
                frontier: fila.items(),
                visited: visitados.clone(),
                path: novo_caminho,
                ..Default::default()
            });
        }
    }

    rastro.registrar(Step {
        phase: Phase::Fail,
        message: "a fila ficou vazia: o objetivo não é alcançável a partir do início".to_string(),
        visited: visitados,
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

struct Quadro {
    no: NodeId,
    caminho: Vec<NodeId>,
    vizinhos: Vec<(NodeId, f64)>,
    proximo: usize,
}

pub fn dfs(
    grafo: &WeightedGraph,
    inicio: &str,
    objetivo: &str,
    limite: Option<usize>,
    rotulo: &str,
) -> Execution {
    let mut rastro = Rastro::new();
    let mut pilha: Vec<Quadro> = vec![Quadro {
        no: inicio.to_string(),
        caminho: vec![inicio.to_string()],
        vizinhos: grafo.vizinhos(inicio).to_vec(),
        proximo: 0,
    }];
    let mut visitados: Vec<NodeId> = vec![inicio.to_string()];
    let mut conjunto: HashSet<NodeId> = HashSet::from([inicio.to_string()]);
    let mut expandidos = 0usize;
    let mut corte = false;

    let descricao_limite = match limite {
        Some(max) => format!(" com profundidade máxima {max}"),
        None => String::new(),
    };
    rastro.registrar(Step {
        phase: Phase::Init,
        node: Some(inicio.to_string()),
        message: format!(
            "{rotulo} desce pelo primeiro vizinho ainda não visitado{descricao_limite}"
        ),
        visited: visitados.clone(),
        call_stack: pilha.iter().map(|q| q.no.clone()).collect(),
        path: pilha[0].caminho.clone(),
        ..Default::default()
    });

    if inicio == objetivo {
        let caminho = vec![inicio.to_string()];
        rastro.registrar(Step {
            phase: Phase::Goal,
            node: Some(inicio.to_string()),
            message: format!("{inicio} já é o objetivo"),
            visited: visitados.clone(),
            call_stack: pilha.iter().map(|q| q.no.clone()).collect(),
            path: caminho.clone(),
            ..Default::default()
        });
        return Execution {
            path: Some(caminho),
            cost: 0.0,
            expanded: 0,
            trace: rastro.trace,
            iterations: vec![],
            corte: false,
        };
    }

    while let Some(topo) = pilha.last() {
        if topo.proximo >= topo.vizinhos.len() {
            let quadro = pilha.pop().expect("acabou de checar que há um topo");
            conjunto.remove(&quadro.no);
            visitados.retain(|v| v != &quadro.no);
            rastro.registrar(Step {
                phase: Phase::Backtrack,
                node: Some(quadro.no.clone()),
                depth: quadro.caminho.len() - 1,
                message: format!(
                    "voltou de {}: todos os vizinhos foram tentados sem sucesso",
                    quadro.no
                ),
                visited: visitados.clone(),
                call_stack: pilha.iter().map(|q| q.no.clone()).collect(),
                path: quadro.caminho.clone(),
                ..Default::default()
            });
            continue;
        }

        let profundidade = pilha.len() - 1;
        let (pai, vizinho, caminho) = {
            let topo = pilha.last_mut().expect("a pilha não pode estar vazia aqui");
            let (vizinho, _) = topo.vizinhos[topo.proximo].clone();
            topo.proximo += 1;
            let mut caminho = topo.caminho.clone();
            caminho.push(vizinho.clone());
            (topo.no.clone(), vizinho, caminho)
        };

        if conjunto.contains(&vizinho) {
            continue;
        }

        if let Some(max) = limite
            && profundidade >= max
        {
            corte = true;
            rastro.registrar(Step {
                phase: Phase::Cut,
                node: Some(vizinho.clone()),
                depth: profundidade + 1,
                message: format!(
                    "podado: {vizinho} ficaria na profundidade {} e o limite é {max}",
                    profundidade + 1
                ),
                visited: visitados.clone(),
                call_stack: pilha.iter().map(|q| q.no.clone()).collect(),
                path: caminho,
                ..Default::default()
            });
            continue;
        }

        let custo_caminho = grafo.custo_do_caminho(&caminho);
        if vizinho == objetivo {
            visitados.push(vizinho.clone());
            let mut pilha_com_objetivo: Vec<NodeId> = pilha.iter().map(|q| q.no.clone()).collect();
            pilha_com_objetivo.push(vizinho.clone());
            rastro.registrar(Step {
                phase: Phase::Goal,
                node: Some(vizinho.clone()),
                g: custo_caminho,
                depth: caminho.len() - 1,
                message: format!(
                    "objetivo {vizinho} alcançado na profundidade {} com custo {custo_caminho}",
                    caminho.len() - 1
                ),
                visited: visitados.clone(),
                call_stack: pilha_com_objetivo,
                path: caminho.clone(),
                ..Default::default()
            });
            return Execution {
                path: Some(caminho),
                cost: custo_caminho,
                expanded: expandidos,
                trace: rastro.trace,
                iterations: vec![],
                corte: false,
            };
        }

        conjunto.insert(vizinho.clone());
        visitados.push(vizinho.clone());
        expandidos += 1;
        let vizinhos = grafo.vizinhos(&vizinho).to_vec();
        pilha.push(Quadro {
            no: vizinho.clone(),
            caminho: caminho.clone(),
            vizinhos,
            proximo: 0,
        });

        rastro.registrar(Step {
            phase: Phase::Expand,
            node: Some(vizinho.clone()),
            g: custo_caminho,
            depth: caminho.len() - 1,
            message: format!(
                "desceu de {pai} para {vizinho} ({} vizinho(s) na pilha de chamada)",
                grafo.vizinhos(&vizinho).len()
            ),
            visited: visitados.clone(),
            call_stack: pilha.iter().map(|q| q.no.clone()).collect(),
            path: caminho,
            ..Default::default()
        });
    }

    let mensagem = if corte {
        "a pilha de chamada esvaziou batendo no limite de profundidade".to_string()
    } else {
        "a pilha de chamada esvaziou: o objetivo não é alcançável a partir do início".to_string()
    };
    rastro.registrar(Step {
        phase: Phase::Fail,
        message: mensagem,
        visited: visitados,
        ..Default::default()
    });

    Execution {
        path: None,
        cost: 0.0,
        expanded: expandidos,
        trace: rastro.trace,
        iterations: vec![],
        corte,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Graph};
    use crate::search::exemplos::nos;

    fn ponderado(directed: bool, edges: Vec<Edge>, nodes: Vec<&str>) -> WeightedGraph {
        let grafo = Graph {
            directed,
            nodes: nos(&nodes),
            edges,
        };
        WeightedGraph::from_graph(&grafo).unwrap()
    }

    #[test]
    fn bfs_desempilha_em_ordem_e_termina_no_objetivo() {
        let g = ponderado(
            false,
            vec![
                Edge::nova("A", "B", 1.0),
                Edge::nova("A", "C", 1.0),
                Edge::nova("B", "D", 1.0),
            ],
            vec!["A", "B", "C", "D"],
        );
        let r = bfs(&g, "A", "D");
        assert_eq!(r.path, Some(vec!["A".into(), "B".into(), "D".into()]));
        let fases: Vec<Phase> = r.trace.iter().map(|p| p.phase).collect();
        assert_eq!(fases[0], Phase::Init);
        assert!(fases.contains(&Phase::Pop));
        assert!(fases.contains(&Phase::Push));
        assert_eq!(fases.last().copied(), Some(Phase::Goal));
    }

    #[test]
    fn bfs_conta_saltos_e_nao_pesos() {
        let g = ponderado(
            false,
            vec![
                Edge::nova("A", "B", 100.0),
                Edge::nova("A", "C", 1.0),
                Edge::nova("B", "T", 1.0),
                Edge::nova("C", "D", 1.0),
                Edge::nova("D", "T", 1.0),
            ],
            vec!["A", "B", "C", "D", "T"],
        );
        let r = bfs(&g, "A", "T");
        assert_eq!(r.path, Some(vec!["A".into(), "B".into(), "T".into()]));
        assert_eq!(r.cost, 101.0);
        let pop = r
            .trace
            .iter()
            .find(|p| p.phase == Phase::Pop && p.node.as_deref() == Some("B"))
            .unwrap();
        assert_eq!(pop.g, 1.0);
    }

    fn beco_sem_saida() -> WeightedGraph {
        ponderado(
            false,
            vec![
                Edge::nova("A", "B", 1.0),
                Edge::nova("B", "C", 1.0),
                Edge::nova("A", "D", 1.0),
                Edge::nova("D", "Z", 1.0),
            ],
            vec!["A", "B", "C", "D", "Z"],
        )
    }

    #[test]
    fn dfs_faz_backtracking_ate_encontrar() {
        let r = dfs(&beco_sem_saida(), "A", "Z", None, "DFS");
        assert_eq!(r.path, Some(vec!["A".into(), "D".into(), "Z".into()]));
        let voltas: Vec<&str> = r
            .trace
            .iter()
            .filter(|p| p.phase == Phase::Backtrack)
            .map(|p| p.node.as_deref().unwrap_or(""))
            .collect();
        assert_eq!(voltas, vec!["C", "B"]);
    }

    #[test]
    fn dfs_desvisita_no_backtracking() {
        let r = dfs(&beco_sem_saida(), "A", "Z", None, "DFS");
        let volta = r
            .trace
            .iter()
            .find(|p| p.phase == Phase::Backtrack)
            .unwrap();
        assert_eq!(volta.node.as_deref(), Some("C"));
        assert!(!volta.visited.contains(&"C".to_string()));
        assert_eq!(volta.call_stack, vec!["A", "B"]);
    }

    #[test]
    fn dfs_com_limite_respeita_a_profundidade() {
        let g = ponderado(
            false,
            vec![
                Edge::nova("N0", "N1", 1.0),
                Edge::nova("N1", "N2", 1.0),
                Edge::nova("N2", "N3", 1.0),
            ],
            vec!["N0", "N1", "N2", "N3"],
        );
        let r = dfs(&g, "N0", "N3", Some(2), "DLS");
        assert_eq!(r.path, None);
        assert!(
            r.trace
                .iter()
                .all(|p| p.call_stack.len().saturating_sub(1) <= 2),
            "a pilha de chamada passou do limite"
        );
        assert!(r.corte);
    }

    #[test]
    fn pilha_de_chamada_aparece_em_cada_passo() {
        let g = ponderado(
            false,
            vec![Edge::nova("A", "B", 1.0), Edge::nova("B", "C", 1.0)],
            vec!["A", "B", "C"],
        );
        let r = dfs(&g, "A", "C", None, "DFS");
        let ultimo = r.trace.last().unwrap();
        assert_eq!(ultimo.call_stack, vec!["A", "B", "C"]);
    }

    #[test]
    fn fila_vazia_encerra_com_falha() {
        let g = ponderado(false, vec![Edge::nova("A", "B", 1.0)], vec!["A", "B", "Z"]);
        let r = bfs(&g, "A", "Z");
        assert_eq!(r.path, None);
        assert_eq!(r.trace.last().map(|p| p.phase), Some(Phase::Fail));
    }

    #[test]
    fn grafo_dirigido_nao_permite_voltar_pela_aresta() {
        let g = ponderado(true, vec![Edge::nova("A", "B", 1.0)], vec!["A", "B", "C"]);
        assert!(bfs(&g, "B", "C").path.is_none());
        assert!(bfs(&g, "A", "B").path.is_some());
    }

    #[test]
    fn inicio_igual_a_objetivo_encerra_no_primeiro_passo() {
        let g = ponderado(false, vec![Edge::nova("A", "B", 1.0)], vec!["A", "B"]);
        let r = bfs(&g, "A", "A");
        assert_eq!(r.path, Some(vec!["A".into()]));
        assert_eq!(r.cost, 0.0);
        assert_eq!(r.expanded, 0);
    }
}
