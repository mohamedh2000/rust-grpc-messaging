use chat::chat_client::ChatClient;
use chat::User;
use socketioxide::{
    extract::SocketRef, SocketIo
};
pub mod chat {
    tonic::include_proto!("chat");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ChatClient::connect("http://[::1]:50051/").await?;

    
    let (_, io) = SocketIo::new_svc();
    io.ns("/", |socket: SocketRef| {
       
        socket.emit("test", "hi").ok();
    });
    
    let request = tonic::Request::new(User {
        user_id: String::from("test"),
    });
    
    let response = client.user_rooms(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}