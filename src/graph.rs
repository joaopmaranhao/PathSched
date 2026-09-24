use std::collections::HashMap;
pub struct Graph {
    adjacencias: HashMap<char, Vec<(char, f64)>>,
}
impl Graph {
    fn new(params: HashMap<char, Vec<(char, f64)>>) -> Self {
        Graph {
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
