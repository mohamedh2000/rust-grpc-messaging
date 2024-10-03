use chat::{
    chat_client::ChatClient, 
    User, 
    FriendRequest
};
use socketioxide::{
    extract::SocketRef, SocketIo
};
pub mod chat {
    tonic::include_proto!("chat");
}

use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ChatClient::connect("http://[::1]:50051/").await?;

    let (_, io) = SocketIo::new_svc();
    io.ns("/", |socket: SocketRef| {
       
        socket.emit("test", "hi").ok();
    });
    
    let ruser_info_request_test = tonic::Request::new(
        User {
            user_id: String::from("test"),
        }
    );

    let friend_request_test = tonic::Request::new(
        FriendRequest {
            sender_user_id: String::from("test_sender_12"),
            receiver_user_id: String::from("test_sender_18"),
            message: String::from("test message"),
            date: Utc::now().timestamp().to_string()
        }
    );
    
    let user_info_response = client.user_info(ruser_info_request_test).await?;
    let friend_request_response = client.send_friend_request(friend_request_test).await?;

    println!("USER_INFO RESPONSE={:?}", user_info_response);
    println!("FRIEND REQUEST RESPONSE={:?}", friend_request_response);

    Ok(())
}
