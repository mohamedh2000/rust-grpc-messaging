use aws_sdk_dynamodb::Client as DynamoClient;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;

use dotenv::dotenv; 

use axum::{Router, serve};

use tracing::info;
use tracing_subscriber::FmtSubscriber;

use socketioxide::SocketIo;

use grpc_messaging::{
    socket::on_connect,
    chat_grpc::{
        ChatService,
        chat::chat_server::ChatServer
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let addr = "[::1]:50051";
    let (layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let dynamodb_client = DynamoClient::new(&config);
    let chat_service =  ChatService::new(dynamodb_client);

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