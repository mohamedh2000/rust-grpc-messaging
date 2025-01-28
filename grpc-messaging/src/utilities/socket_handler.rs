use std::collections::HashMap;

use crate::utilities::dynamo_operations::{put_dynamodb, build_dynamo_client, query_dynamodb};
use super::{
    info, 
    Data, 
    SocketRef,
    env
};
use aws_sdk_dynamodb::types::AttributeValue;
use uuid::{self, Uuid};

pub async fn on_connect(s: SocketRef) {
    info!("socket connected: {}", s.id);

    s.on("join", |s: SocketRef, Data::<String>(room)| {
        info!("im in join room namespace");
        info!("joined room {}", room);
        s.join(room.clone())
        .ok();
    });

    s.on(
        "new_message", |s: SocketRef, Data::<(String, String)>((room, message))| {
            info!("im in new_message");
            info!("im in new_message, room: {:?}, message: {:?} ", room, message);
            new_message_handler(s, room, message)
            //TODO: Make it so that this just broadcasts to everyone in the room 
            //on disconnect it will send each message to the backend and it will check the database
            //to see what is new and what isn't and then upload to the db
        }
    );

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

pub async fn new_message_handler(s: SocketRef, room: String, message: String) {
    info!("im in here");
    println!("message being sent to room: {:?}, message: {:?}", room.clone(), message.clone());
    let message_table_env = &env::var("MESSAGE_TABLE").unwrap();

    s.within(room.to_string()).emit("new message", message.clone()).ok();
    let client: aws_sdk_dynamodb::Client = build_dynamo_client().await; //TODO COME BACK TO THIS, SHOUL 

    let table_entry = put_dynamodb(
        client, 
        message_table_env, 
        vec![
            (String::from("message_id"), Uuid::new_v4().to_string()),
            (String::from("room_id"), String::from(room) /*msg.room_id.clone()*/),
            (String::from("message"), String::from(message) /*msg.message.clone()*/),
            (String::from("userId"), String::from("test")/*msg.user_id.clone()*/),
            (String::from("date"), String::from("tuesday")),
        ]
    ).await;

    match table_entry {
        Ok(_) => info!("new message entry was entered"),
        Err(e) => info!("new error message: {}", e.to_string())
    } 
}
