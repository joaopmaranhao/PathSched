use super::{Execution, Iteration, WeightedGraph, dfs_and_bfs};
use crate::graph::NodeId;

pub fn dls(
    grafo: &WeightedGraph,
    inicio: &str,
    objetivo: &str,
    limite: Option<usize>,
) -> Execution {
    dfs_and_bfs::dfs(grafo, inicio, objetivo, limite, "DLS")
}

pub fn ids(grafo: &WeightedGraph, inicio: &str, objetivo: &str, max_iteracoes: usize) -> Execution {
    let mut trace = Vec::new();
    let mut iteracoes: Vec<Iteration> = Vec::new();
    let mut expandidos = 0usize;
    let mut resultado: Option<(Vec<NodeId>, f64)> = None;
    let mut corte = false;

    for (i, limite) in (0..=max_iteracoes).enumerate() {
        let execucao = dls(grafo, inicio, objetivo, Some(limite));
        let inicio_trace = trace.len();
        let passos = execucao.trace.len();
        let achou = execucao.path.is_some();

        for (j, passo) in execucao.trace.into_iter().enumerate() {
            let mut passo = passo;
            passo.index = inicio_trace + j;
            passo.iteration = i + 1;
            passo.limit = Some(limite);
            trace.push(passo);
        }

        iteracoes.push(Iteration {
            limit: limite,
            steps: passos,
            found: achou,
        });

        expandidos += execucao.expanded;
        corte = execucao.corte;
        if let Some(caminho) = execucao.path {
            resultado = Some((caminho, execucao.cost));
            break;
        }
        if !execucao.corte {
            break;
        }
    }

    let (path, cost) = match resultado {
        Some((caminho, custo)) => (Some(caminho), custo),
        None => (None, 0.0),
    };

    Execution {
        path,
        cost,
        expanded: expandidos,
        trace,
        iterations: iteracoes,
        corte,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::exemplos::{grafo_classico, grafo_linhado, grafo_sem_saida};
    use crate::search::{MAX_ITERACOES_PADRAO, Phase};

    #[test]
    fn dls_encontra_com_limite_suficiente() {
        let g = WeightedGraph::from_graph(&grafo_classico()).unwrap();
        let r = dls(&g, "A", "E", Some(3));
        assert_eq!(
            r.path,
            Some(vec!["A".into(), "B".into(), "C".into(), "E".into()])
        );
        assert_eq!(r.cost, 15.0);
    }

    #[test]
    fn dls_com_limite_zero_sempre_corta() {
        let g = WeightedGraph::from_graph(&grafo_classico()).unwrap();
        let r = dls(&g, "A", "E", Some(0));
        assert_eq!(r.path, None);
        assert!(r.corte);
    }

    #[test]
    fn ids_repete_a_busca_com_limites_crescentes() {
        let g = WeightedGraph::from_graph(&grafo_linhado(5)).unwrap();
        let r = ids(&g, "N0", "N4", MAX_ITERACOES_PADRAO);
        assert!(r.path.is_some());
        assert_eq!(r.iterations.len(), 5, "limites 0, 1, 2, 3 e 4");
        let limites: Vec<usize> = r.iterations.iter().map(|i| i.limit).collect();
        assert_eq!(limites, vec![0, 1, 2, 3, 4]);
        assert!(r.iterations[4].found);
        assert!(r.iterations[..4].iter().all(|i| !i.found));
    }

    #[test]
    fn ids_marca_iteracao_e_limite_em_cada_passo() {
        let g = WeightedGraph::from_graph(&grafo_linhado(4)).unwrap();
        let r = ids(&g, "N0", "N3", MAX_ITERACOES_PADRAO);
        for passo in &r.trace {
            assert!(passo.iteration >= 1);
            assert_eq!(passo.limit, Some(passo.iteration - 1));
            assert!(passo.index < r.trace.len());
        }
    }

    #[test]
    fn ids_para_quando_o_espaco_acaba() {
        let g = WeightedGraph::from_graph(&grafo_sem_saida()).unwrap();
        let r = ids(&g, "A", "Z", MAX_ITERACOES_PADRAO);
        assert_eq!(r.path, None);
        // os limites 0 e 1 batem no limite; com 2 a busca percorre A-B-C e desiste
        assert_eq!(r.iterations.len(), 3);
        assert!(!r.corte);
        assert_eq!(r.trace.last().map(|p| p.phase), Some(Phase::Fail));
    }

    #[test]
    fn ids_respeita_o_maximo_de_iteracoes() {
        let g = WeightedGraph::from_graph(&grafo_linhado(30)).unwrap();
        let r = ids(&g, "N0", "N29", 3);
        assert_eq!(r.path, None);
        assert_eq!(r.iterations.len(), 4);
    }

    #[test]
    fn ids_conta_nos_expandidos_de_todas_as_iteracoes() {
        let g = WeightedGraph::from_graph(&grafo_linhado(5)).unwrap();
        // os limites 0, 1, 2 e 3 expandem 0, 1, 2 e 3 nós; na última (limite 4) são 3,
        // porque o objetivo não conta como nó expandido
        let r = ids(&g, "N0", "N4", MAX_ITERACOES_PADRAO);
        assert_eq!(r.expanded, 1 + 2 + 3 + 3);
    }
}
