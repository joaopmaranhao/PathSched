// Demonstração no terminal: DLS, IDS, UCS, BFS e DFS no mesmo grafo.
// rode com: cargo run --example busca
use pathsched::graph::{Edge, Graph, WeightedGraph};
use std::collections::HashMap;

fn main() {
    let nos = ["A", "B", "C", "D", "E"];
    let arestas = [
        Edge::nova("A", "B", 4.0),
        Edge::nova("A", "C", 2.0),
        Edge::nova("B", "C", 1.0),
        Edge::nova("B", "D", 2.0),
        Edge::nova("C", "D", 5.0),
        Edge::nova("C", "E", 10.0),
        Edge::nova("D", "E", 1.0),
    ];

    println!("Grafo: {nos:?}");
    for aresta in &arestas {
        println!(
            "  {} -> {} (custo {})",
            aresta.from, aresta.to, aresta.weight
        );
    }

    let grafo_json = serde_json::json!({
        "directed": false,
        "nodes": nos,
        "edges": arestas
            .iter()
            .map(|a| serde_json::json!({ "from": a.from, "to": a.to, "weight": a.weight }))
            .collect::<Vec<_>>(),
    });
    let ponderado =
        WeightedGraph::from_graph(&serde_json::from_value::<Graph>(grafo_json).unwrap()).unwrap();

    println!("\nDLS com limite 2:  {:?}", ponderado.dls("A", "E", 2));
    println!("DLS com limite 3:  {:?}", ponderado.dls("A", "E", 3));
    println!("IDS (20 iterações): {:?}", ponderado.ids("A", "E", 20));
    println!("UCS:                {:?}", ponderado.ucs("A", "E"));
    println!("BFS:                {:?}", ponderado.bfs("A", "E"));
    println!("DFS:                {:?}", ponderado.dfs("A", "E"));

    if let Some(caminho) = ponderado.ucs("A", "E") {
        println!(
            "\nUCS achou {:?} com custo {} (o mínimo do grafo)",
            caminho,
            ponderado.custo_do_caminho(&caminho)
        );
    }

    let mut adjacencias: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    for id in nos {
        adjacencias.insert(id.to_string(), Vec::new());
    }
    for aresta in &arestas {
        adjacencias
            .get_mut(&aresta.from)
            .unwrap()
            .push((aresta.to.clone(), aresta.weight));
    }
    let por_adjacencias = WeightedGraph::new(adjacencias);
    // `new` usa o mapa literalmente (seta A -> B e não volta), então o custo sai diferente
    println!(
        "com adjacências construídas à mão (sentido único): {:?}",
        por_adjacencias.ucs("A", "E")
    );
}
