use pathsched::web;
use std::net::SocketAddr;
use tokio::net::TcpListener;

const PORTA_PADRAO: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let porta = porta_configurada();
    let endereco = SocketAddr::from(([127, 0, 0, 1], porta));
    let url = format!("http://{endereco}");

    let listener = TcpListener::bind(endereco).await?;
    println!("PathSched — visualizador de busca em grafos");
    println!("  servidor em {url}");
    println!("  o grafo é montado no navegador (exemplo pronto, JSON ou editor visual)");
    println!("  encerrar com ctrl-c");

    if abrir_navegador() {
        let url_para_abrir = url.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            if let Err(erro) = open::that(&url_para_abrir) {
                eprintln!(
                    "não consegui abrir o navegador: {erro} — abra {url_para_abrir} no navegador"
                );
            }
        });
    }

    axum::serve(listener, web::router())
        .with_graceful_shutdown(shutdown())
        .await?;
    println!("servidor encerrado");
    Ok(())
}

fn porta_configurada() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|valor| valor.parse().ok())
        .unwrap_or(PORTA_PADRAO)
}

fn abrir_navegador() -> bool {
    let flag = std::env::args().any(|arg| arg == "--no-open");
    let env = std::env::var("PATHSCHED_NO_OPEN").is_ok();
    !flag && !env
}

async fn shutdown() {
    if tokio::signal::ctrl_c().await.is_ok() {
        println!("\ninterrompido pelo usuário");
    }
}
