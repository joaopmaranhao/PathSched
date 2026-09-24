use std::collections::HashMap;
pub struct WeightedGraph {
    adjacencias: HashMap<char, Vec<(char, f64)>>,
}
impl WeightedGraph {
    fn new(params: HashMap<char, Vec<(char, f64)>>) -> Self {
        WeightedGraph {
            adjacencias: params,
        }
    }
    fn bfs(self) -> Option<Vec<char>> {
        todo!()
    }
    fn dfs(self) -> Option<Vec<char>> {
        todo!()
    }
    fn ucs(self) -> Option<Vec<char>> {
        todo!()
    }
    fn ids(self) -> Option<Vec<char>> {
        todo!()
    }
    fn dls(self) -> Option<Vec<char>> {
        todo!()
    }
}
