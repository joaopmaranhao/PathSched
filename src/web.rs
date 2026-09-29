use crate::graph::ErroGrafo;
use crate::search::{Requisicao, SearchResult};
use axum::extract::rejection::JsonRejection;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use include_dir::{Dir, include_dir};

static APP: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/www");

pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        .route("/api/search", post(search))
        .fallback(asset)
}

async fn index() -> Response {
    asset_por_caminho("/index.html")
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "service": "pathsched" }))
}

async fn search(
    requisicao: Result<Json<Requisicao>, JsonRejection>,
) -> Result<Json<SearchResult>, ErroApi> {
    let Json(requisicao) = requisicao.map_err(|erro| ErroApi {
        status: StatusCode::BAD_REQUEST,
        corpo: ErroGrafo::novo(format!("JSON inválido: {erro}")),
    })?;

    let resultado = crate::search::executar(&requisicao).map_err(ErroApi::requisicao)?;
    Ok(Json(resultado))
}

struct ErroApi {
    status: StatusCode,
    corpo: ErroGrafo,
}

impl ErroApi {
    fn requisicao(erro: ErroGrafo) -> Self {
        ErroApi {
            status: StatusCode::BAD_REQUEST,
            corpo: erro,
        }
    }
}

impl IntoResponse for ErroApi {
    fn into_response(self) -> Response {
        (self.status, Json(self.corpo)).into_response()
    }
}

async fn asset(caminho: axum::http::Uri) -> Response {
    asset_por_caminho(caminho.path())
}

fn asset_por_caminho(caminho: &str) -> Response {
    let caminho = caminho.trim_start_matches('/');
    match APP.get_file(caminho) {
        Some(arquivo) => {
            let tipo = tipo_do_arquivo(caminho);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, tipo)],
                arquivo.contents(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "arquivo não encontrado").into_response(),
    }
}

fn tipo_do_arquivo(caminho: &str) -> &'static str {
    match caminho.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Node};
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn grafo_json() -> String {
        serde_json::json!({
            "directed": false,
            "nodes": ["A", "B", "C", "D", "E"],
            "edges": [
                {"from": "A", "to": "B", "weight": 4.0},
                {"from": "A", "to": "C", "weight": 2.0},
                {"from": "B", "to": "C", "weight": 1.0},
                {"from": "B", "to": "D", "weight": 2.0},
                {"from": "C", "to": "D", "weight": 5.0},
                {"from": "C", "to": "E", "weight": 10.0},
                {"from": "D", "to": "E", "weight": 1.0}
            ]
        })
        .to_string()
    }

    async fn corpo(resposta: Response) -> String {
        let bytes = axum::body::to_bytes(resposta.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    async fn get(caminho: &str) -> Response {
        router()
            .oneshot(Request::builder().uri(caminho).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn post_busca(algoritmo: &str, inicio: &str, objetivo: &str) -> Response {
        let payload = serde_json::json!({
            "graph": serde_json::from_str::<serde_json::Value>(&grafo_json()).unwrap(),
            "algorithm": algoritmo,
            "start": inicio,
            "goal": objetivo,
            "max_iterations": 20
        });
        router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/search")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn health_responde_ok() {
        let resposta = get("/api/health").await;
        assert_eq!(resposta.status(), StatusCode::OK);
        assert!(corpo(resposta).await.contains("\"ok\":true"));
    }

    #[tokio::test]
    async fn index_e_servido_com_html() {
        let resposta = get("/").await;
        assert_eq!(resposta.status(), StatusCode::OK);
        let tipo = resposta
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(tipo.starts_with("text/html"));
        assert!(corpo(resposta).await.contains("<!DOCTYPE html>"));
    }

    #[tokio::test]
    async fn assets_do_frontend_sao_servidos() {
        for caminho in [
            "/app.js",
            "/style.css",
            "/presets.js",
            "/render.js",
            "/player.js",
            "/graph.js",
        ] {
            let resposta = get(caminho).await;
            assert_eq!(resposta.status(), StatusCode::OK, "faltou {caminho}");
            assert!(!corpo(resposta).await.is_empty());
        }
    }

    #[tokio::test]
    async fn arquivo_inexistente_responde_404() {
        assert_eq!(get("/nao-existe.js").await.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn busca_ucs_devolve_o_caminho_e_o_rastro() {
        let resposta = post_busca("ucs", "A", "E").await;
        assert_eq!(resposta.status(), StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&corpo(resposta).await).unwrap();
        assert_eq!(json["found"], true);
        assert_eq!(json["path"], serde_json::json!(["A", "C", "B", "D", "E"]));
        assert_eq!(json["total_cost"], 6.0);
        assert_eq!(json["algorithm"], "ucs");
        assert!(json["steps"].as_u64().unwrap() > 0);
        assert!(!json["trace"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn busca_ids_devolve_as_iteracoes() {
        let resposta = post_busca("ids", "A", "E").await;
        let json: serde_json::Value = serde_json::from_str(&corpo(resposta).await).unwrap();
        assert_eq!(json["found"], true);
        assert_eq!(json["iterations"][0]["limit"], 0);
        assert!(json["iterations"].as_array().unwrap().len() > 1);
    }

    #[tokio::test]
    async fn busca_com_no_inexistente_responde_400() {
        let resposta = post_busca("ucs", "Z", "E").await;
        assert_eq!(resposta.status(), StatusCode::BAD_REQUEST);
        assert!(corpo(resposta).await.contains("Z"));
    }

    #[tokio::test]
    async fn busca_com_json_invalido_responde_400() {
        let resposta = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/search")
                    .header("content-type", "application/json")
                    .body(Body::from("{ não é json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resposta.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn grafo_vazio_responde_400() {
        let payload = serde_json::json!({
            "graph": {"directed": false, "nodes": [], "edges": []},
            "algorithm": "bfs",
            "start": "A",
            "goal": "B"
        });
        let resposta = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/search")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resposta.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn os_assets_estao_embutidos_no_binario() {
        assert!(APP.get_file("index.html").is_some());
        assert!(APP.get_file("app.js").is_some());
    }

    #[test]
    fn nos_com_rotulo_simples_sao_aceitos_pela_api() {
        let grafo: serde_json::Value = serde_json::from_str(&grafo_json()).unwrap();
        let deserializado: crate::graph::Graph = serde_json::from_value(grafo).unwrap();
        deserializado.validar().unwrap();
        assert_eq!(deserializado.nodes.len(), 5);
        assert!(deserializado.nodes.iter().all(|n| n.label.is_none()));
        let _ = Node::novo("A");
        let _ = Edge::nova("A", "B", 1.0);
    }
}
