# PathSched

Visualizador de busca em largura em grafos, feito em Rust (Axum) com interface em
HTML, CSS e JavaScript puros — sem framework e sem etapa de build no produto.

O grafo é montado no navegador (presets prontos, editor JSON ou editor visual no
próprio SVG) e enviado ao backend, que executa o algoritmo e devolve o caminho,
o custo e o **rastro passo a passo** da busca. A interface reproduz esse rastro
como uma animação, mostrando a fila/pilha, os nós fechados, a pilha de chamada e
o log de cada passo.

## Algoritmos

| Chave  | Algoritmo | Como expande                                        |
| ------ | --------- | --------------------------------------------------- |
| `bfs`  | Busca em largura | por número de saltos (ignora o peso das arestas) |
| `dfs`  | Busca em profundidade | empilha os vizinhos, com backtracking          |
| `dls`  | Busca em profundidade limitada | como o DFS, até o limite `limit`             |
| `ids`  | Busca iterativa em profundidade | repete o DLS com limites `0..=max_iterations`  |
| `ucs`  | Busca de custo uniforme | ordena a fronteira por custo acumulado `g`      |

No preset clássico o BFS e o UCS discordam de propósito: o caminho mais curto em
saltos (`A → C → E`, custo 12) não é o mais barato (`A → C → B → D → E`, custo 6).

## Como executar

```sh
cargo run
```

O servidor sobe em `http://127.0.0.1:3000` e abre o navegador sozinho.

Opções:

- `PORT=8080 cargo run` — troca a porta.
- `cargo run -- --no-open` ou `PATHSCHED_NO_OPEN=1 cargo run` — não abre o navegador.
- `cargo run --example busca` — resumo das buscas no terminal, sem navegador.
- `cargo test` — testes dos algoritmos, do grafo e das rotas HTTP.

Os arquivos de `src/www` são embutidos no binário em tempo de compilação; não há
nada para servir em separado. Depois de editar o frontend, é preciso recompilar
para que o navegador receba a versão nova.

## Interface

- **Presets**: grafos de exemplo (clássico de 5 nós, mapa da Romênia, custo x
  saltos, corrida profunda para o IDS e um grafo dirigido).
- **Editor visual**: ative o modo de edição, clique no palco para criar um nó e
  clique em dois nós para ligar uma aresta; duplo clique renomeia um nó. Clique
  com o botão direito para remover.
- **JSON**: reflita o grafo atual, edite à mão e clique em *Aplicar JSON*. Erros
  de sintaxe ou de validação aparecem ao lado do botão.
- **Reprodução**: primeiro passo, voltar, tocar/pausar, próximo, último passo e
  controle de velocidade; o log também aceita clique para saltar a um passo.
- **Painel lateral**: situação, caminho, custo, nós expandidos, fase atual,
  profundidade, iteração e limite, fila/pilha, fechados, pilha de chamada e o log.
  No IDS aparece ainda a barra de iterações, clicável para pular ao início de cada
  repetição.

## API

### `POST /api/search`

```json
{
  "graph": {
    "directed": false,
    "nodes": ["A", "B", "C"],
    "edges": [
      { "from": "A", "to": "B", "weight": 3 },
      { "from": "B", "to": "C", "weight": 4 }
    ]
  },
  "algorithm": "ucs",
  "start": "A",
  "goal": "C",
  "max_iterations": 20,
  "limit": null
}
```

- `nodes` aceita `"A"` ou `{ "id": "A", "label": "Arad", "x": 0, "y": 0 }`.
- `weight` é o custo da aresta (>= 0). Arestas de grafo não dirigido valem nos
  dois sentidos; em grafo dirigido, só `from → to`.
- `max_iterations` só é usado pelo IDS; `limit` só pelo DLS.
- `algorithm` fora da lista acima devolve `400`.

Resposta (`SearchResult`):

```json
{
  "algorithm": "ucs",
  "start": "A",
  "goal": "C",
  "found": true,
  "path": ["A", "B", "C"],
  "total_cost": 7,
  "expanded": 2,
  "steps": 9,
  "exhausted": false,
  "iterations": [],
  "trace": [
    {
      "index": 0,
      "phase": "init",
      "node": "A",
      "g": 0,
      "depth": 0,
      "iteration": 0,
      "limit": null,
      "message": "fila de prioridade com A (g = 0). O menor custo sai primeiro",
      "frontier": [{ "node": "A", "g": 0, "weight": 0, "path": ["A"], "seq": 0 }],
      "visited": [],
      "call_stack": [],
      "path": []
    }
  ]
}
```

Fases possíveis: `init`, `push`, `pop`, `expand`, `visit`, `goal`, `backtrack`,
`cut`, `fail`. No IDS, cada passo traz `iteration` e `limit` da repetição.

### `GET /api/health`

```json
{ "ok": true, "service": "pathsched" }
```

Erros de entrada ou de grafo saem como `400` com `{"erro": "..."}`.

Exemplo rápido:

```sh
curl -s localhost:3000/api/search -H 'content-type: application/json' -d '{
  "graph": {"directed": false, "nodes": ["A","B","C"],
            "edges": [{"from":"A","to":"B","weight":3},{"from":"B","to":"C","weight":4}]},
  "algorithm": "ucs", "start": "A", "goal": "C"}'
```

## Estrutura

```
src/
  main.rs        servidor, porta, abertura do navegador, encerramento
  lib.rs         módulos públicos da biblioteca
  web.rs         rotas Axum e assets embutidos
  graph.rs       grafo, validação e adjacências ponderadas
  search/        tipos do rastro, despacho e os algoritmos
  www/           interface (index.html, style.css, app.js, graph.js,
                 render.js, player.js, presets.js)
examples/
  busca.rs       comparação dos algoritmos no terminal
```

Os rótulos dos nós são `String`: podem ser `A`, `Arad` ou qualquer outro nome.
