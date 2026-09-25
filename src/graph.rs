use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;
#[derive(Debug, Clone)]
struct Entry {
    cost: f64,
    node: char,
    path: Vec<char>,
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
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
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}
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
    pub fn ucs(
        &self,
        no_inicio: char, 
        no_objetivo: char
    ) -> Option<Vec<char>> {
        
        let mut priority_queue : BinaryHeap<Entry> = BinaryHeap::new();
        let mut visited : HashSet<char> = HashSet::new();

        priority_queue.push(Entry {
            cost: 0.0,
            node : no_inicio,
            path: vec![no_inicio],
         });

        while let Some(Entry {cost, node, path}) = priority_queue.pop() {
            if node == no_objetivo {
                return Some(path);
            }
            if visited.contains(&node){
                continue;
            }
            visited.insert(node);

            if let Some(neighbors) = self.adjacencias.get(&node) {
                for &(neighbor, weight) in neighbors {
                    if !visited.contains(&neighbor){
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        priority_queue.push(
                            Entry {
                                cost : cost + weight,
                                node : neighbor,
                                path : new_path,
                            });
                    }
                }
            }
        }
        None       
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
