use std::collections::{HashMap, HashSet};
#[derive(Debug)]
pub struct WeightedGraph {
    adjacencias: HashMap<char, Vec<(char, f64)>>,
}
impl WeightedGraph {
    pub fn new(params: HashMap<char, Vec<(char, f64)>>) -> Self {
        WeightedGraph {
            adjacencias: params,
        }
    }
    pub fn bfs(&self) -> Option<Vec<char>> {
        todo!()
    }
    pub fn dfs(&self, no_inicio: char, no_objetivo: char) -> Option<Vec<char>> {
        let mut corte = false;
        let mut visitados = HashSet::new();
        self.dls(
            no_inicio,
            no_objetivo,
            usize::MAX,
            &mut corte,
            &mut visitados,
        )
    }
    pub fn ucs(&self) -> Option<Vec<char>> {
        todo!()
    }
    pub fn ids(
        &self,
        no_inicio: char,
        no_objetivo: char,
        max_iterations: usize,
    ) -> Option<Vec<char>> {
        let mut limite: usize = 0;
        while limite <= max_iterations {
            let mut corte = false;
            let mut visitados = HashSet::new();
            if let Some(x) = self.dls(no_inicio, no_objetivo, limite, &mut corte, &mut visitados) {
                return Some(x);
            }
            if !corte {
                return None;
            }
            limite += 1;
        }
        None
    }
    pub fn dls(
        &self,
        no_inicio: char,
        no_objetivo: char,
        limite: usize,
        corte: &mut bool,
        visitados: &mut HashSet<char>,
    ) -> Option<Vec<char>> {
        if no_inicio == no_objetivo {
            return Some(vec![no_inicio]);
        }
        if limite == 0 {
            *corte = true;
            return None;
        }
        visitados.insert(no_inicio);
        if let Some(vizinhos) = self.adjacencias.get(&no_inicio) {
            for &(vizinho, _custo) in vizinhos {
                if !visitados.contains(&vizinho)
                    && let Some(mut caminho) =
                        self.dls(vizinho, no_objetivo, limite - 1, corte, visitados)
                {
                    caminho.insert(0, no_inicio);
                    return Some(caminho);
                }
            }
        }
        visitados.remove(&no_inicio);
        None
    }
}
