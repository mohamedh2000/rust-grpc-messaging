mod utilities;
use utilities::{
    chat_grpc::{
        chat::chat_server::ChatServer,
        create_service 
    }, 
    socket_handler::on_connect
};

use dotenv::dotenv; 

use axum::{Router, serve};

use tracing::info;
use tracing_subscriber::FmtSubscriber;

use socketioxide::SocketIo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let addr = "[::1]:50051";
    let (layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);

    let chat_service = create_service().await; 

    println!("ChatServer listening on {}", addr);

    let app = Router::new()
    .route(
        "/*rpc",
        axum::routing::any_service(ChatServer::new(chat_service))
    )
    .with_state(io)
    .layer(
        tower::ServiceBuilder::new()
            .layer(tower_http::cors::CorsLayer::permissive())
            .layer(layer)
    );

    info!("Starting server");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();

    Ok(())
}