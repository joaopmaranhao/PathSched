// PathSched: buscas em grafos com custo e rastro passo a passo.
// A API pública (WeightedGraph, search, web) também é usada em examples/ e nos testes,
// por isso o crate não reclama de itens que o binário não chama.
#![allow(dead_code)]

pub mod graph;
pub mod search;
pub mod web;
