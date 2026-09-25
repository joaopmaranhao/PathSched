use crate::graph::WeightedGraph;
use std::collections::HashMap;
mod graph;

fn main() {
    let mut adjacencias: HashMap<char, Vec<(char, f64)>> = HashMap::new();

    adjacencias.insert('A', vec![('B', 4.0), ('C', 2.0)]);
    adjacencias.insert('B', vec![('D', 2.0), ('C', 1.0)]);
    adjacencias.insert('C', vec![('D', 5.0), ('E', 10.0)]);
    adjacencias.insert('D', vec![('E', 1.0)]);
    adjacencias.insert('E', vec![]);
    let g = WeightedGraph::new(adjacencias);
    let mut corte = false;
    let mut visitados = std::collections::HashSet::new();
    let res_dls = g.dls('A', 'E', 2, &mut corte, &mut visitados);
    let res_ucs = g.ucs('A', 'E');
    println!("{res_dls:?}");
    println!("{res_ucs:?}");
}
