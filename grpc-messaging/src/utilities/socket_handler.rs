use crate::utilities::dynamo_operations::{put_dynamodb, build_dynamo_client};
use super::{
    info, 
    Data, 
    SocketRef,
    env
};

pub async fn on_connect(s: SocketRef) {
    info!("socket connected: {}", s.id);

    s.on("join", |s: SocketRef, Data::<String>(room)| {
        info!("joined room {}", room);
        s.join(room)
        .ok();
    });

    s.on(
        "new message", message_handler
    );

    s.on("test", |s: SocketRef| {
        info!("Im in test");
        s.broadcast()
            .emit("test", "Testing!").ok();
    });

    s.on(
        "typing", |s: SocketRef| {
            s.broadcast()
                .emit("typing", "typing")
                .ok();
        }
    );

    s.on_disconnect(|s: SocketRef| {
        s.broadcast().emit("user left", "gone")
        .ok();
    })

}

pub async fn message_handler(s: SocketRef, room: Data::<String>) {
    println!("rooms connected: {:?}",s.rooms());
    let message_table_env = &env::var("MESSAGE_TABLE").unwrap();

    s.within(room.to_string()).emit("new message", "hi").ok();
    let client = build_dynamo_client().await; //TODO COME BACK TO THIS, SHOULDN'T BE REINITIALIZED THIS MUCH

    let table_entry = put_dynamodb(
        client, 
        message_table_env, 
        vec![
            (String::from("userId"), String::from("test")/*msg.user_id.clone()*/),
            (String::from("roomId"), String::from("test") /*msg.room_id.clone()*/),
            (String::from("username"), String::from("test") /*msg.username.clone()*/),
            (String::from("message"), String::from("test") /*msg.message.clone()*/),
        ]
    ).await;

    match table_entry {
        Ok(_) => info!("new message entry was entered"),
        Err(e) => info!("new error message: {}", e.to_string())
    }
}