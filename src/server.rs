use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes};
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;

use chat::chat_server::{Chat, ChatServer};
use chat::{User, UserRoomResponse, Room, ChatMessage, Empty, RoomRequest, FriendRequest};

use std::{collections::HashMap, sync::{Arc, Mutex}};
use std::convert::From;
use dotenv::dotenv; 
use std::env;

use axum::{Router, serve};
use tonic::{Request, Response, Status, Code};
use tokio::sync::broadcast;

use socketioxide::{
    extract::{Data, SocketRef},
    SocketIo,
};
use tracing::info;
use tracing_subscriber::FmtSubscriber;



type MessageProducer = broadcast::Sender<ChatMessage>;
type MessageConsumer = broadcast::Receiver<ChatMessage>;

pub mod chat { 
    tonic::include_proto!("chat");
}

struct RoomChannel {
    producer: MessageProducer, 
    consumer: MessageConsumer,
}

pub struct ChatService {
    dynamodb_client: DynamoClient,
    rooms: Arc<Mutex<HashMap<String, RoomChannel>>>,
}

impl ChatService {
    fn new(dynamodb_client : DynamoClient) -> Self {
        ChatService{
            dynamodb_client, 
            rooms: Arc::new(
                Mutex::new(
                    HashMap::new()
                )
            ),
        }
    }
}

#[tonic::async_trait]
impl Chat for ChatService {

    //TODO
    async fn request_room(
        &self,
        request: Request<RoomRequest>,
    ) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty{}))
    }

    async fn request_friend(
        &self, 
        request: Request<FriendRequest>,
    ) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty{}))
    }

    async fn user_rooms(
        &self, 
        request: Request<User>,
    ) -> Result<Response<UserRoomResponse>, Status> { 
        println!("Got a request from {:?}", request.remote_addr());

        let user_table_env = &env::var("USER_TABLE").unwrap();
        let room_table_env = &env::var("ROOM_TABLE").unwrap(); 

        let user_id: String = request.into_inner().user_id;
        let results = query_dynamodb(
            &self.dynamodb_client, 
            user_table_env, 
            vec![(String::from("userId"), user_id)]).await.unwrap();
        let results = results.get( user_table_env).unwrap();

        let user_rooms = &results[0];
        let rooms_vec = AttributeValue::as_l(user_rooms.get("rooms").unwrap()).unwrap();
        let rooms_vec = rooms_vec.iter().map(|room| {
            (String::from("roomId"), String::from(AttributeValue::as_n(room).unwrap()))
        }).collect::<Vec<_>>();

        match UserRoomResponse::get_batch_results(rooms_vec,  room_table_env, &self.dynamodb_client).await { 
            Ok(user_rooms) => Ok(Response::new(user_rooms)),
            Err(err) => Err(Status::new(Code::Aborted, err.to_string())),
        }
    }

    //TODO: Might not be needed
    async fn send_message(
        &self, 
        request: Request<ChatMessage>,
    ) -> Result<Response<Empty>, Status> {
        let chat_message = request.into_inner();
        let room_id = chat_message.room_id.clone();
        let message_table_env = &env::var("MESSAGE_TABLE").unwrap();
        
        let results = put_dynamodb(&self.dynamodb_client, message_table_env, 
            vec![
            (String::from("userId"), chat_message.user_id.clone()),
            (String::from("roomId"), chat_message.room_id.clone()),
            (String::from("username"), chat_message.username.clone()),
            (String::from("message"), chat_message.message.clone()),
            ]
        ).await;

        match results {
            Ok(_) => {
                let room_map = self.rooms.lock().unwrap();  
                let room_channel = room_map.get(&room_id).unwrap(); 
                room_channel.producer.send(chat_message);
                Ok(Response::new(Empty{}))
            },
            Err(e) => Err(Status::new(Code::Aborted, e.to_string())),
        }
    }

}

trait DynamoResultConversions<T, U> {
    fn convert_to_message(to_convert: &U) -> T;
}

impl DynamoDbQuery<UserRoomResponse> for UserRoomResponse {}

trait DynamoDbQuery<T> where T: DynamoResultConversions<T, Vec<HashMap<String, AttributeValue>>> {
    //makes a batch get request to dynamodb and then converts data to the passed in type T
    async fn get_batch_results( map: Vec<(String, String)>, table_name:&str, client: &DynamoClient) -> Result<T, aws_sdk_dynamodb::Error> {
        let result = query_dynamodb(&client, table_name, map ).await?;
        let rooms_metadata = result.get(table_name).unwrap();
        Ok(T::convert_to_message(rooms_metadata))
    }
}

impl DynamoResultConversions<UserRoomResponse, Vec<HashMap<String, AttributeValue>>> for UserRoomResponse {
    fn convert_to_message(to_convert:&Vec<HashMap<String, AttributeValue>>) -> Self {
        let rooms_metadata = to_convert.iter().map(|room| {
            Room::convert_to_message(room)
        }).collect::<Vec<_>>();
        UserRoomResponse {
            rooms:rooms_metadata
        }
    }
}

impl DynamoResultConversions<Room, HashMap<String, AttributeValue>> for Room {
    fn convert_to_message(to_convert: &HashMap<String, AttributeValue>) -> Self {
        Room {
            room_id: AttributeValue::as_s(to_convert.get("roomId").unwrap()).unwrap().parse::<u64>().unwrap(),
            room_name: String::from(AttributeValue::as_s(to_convert.get("roomName").unwrap()).unwrap()),
        }
    }
}

pub async fn query_dynamodb( //with the result make sure to get table_name and then you can iterate over the values
    client: &DynamoClient,
    table_name: &str,
    query_params: Vec<(String, String)>
) -> Result<HashMap<String, Vec<HashMap<String, AttributeValue>>>, aws_sdk_dynamodb::Error> {

    let mut vector_attributes: Vec<HashMap<String, AttributeValue>> = vec![];
    for params in query_params {
        vector_attributes.push(
            HashMap::from([(
                params.0, 
                AttributeValue::S(params.1),
            )])
        )
    }; 
    let dynamo_query = KeysAndAttributes::builder().set_keys(Some(vector_attributes)).build().unwrap();
    let result = client.batch_get_item().request_items(table_name, dynamo_query).send().await?;

    Ok(result.responses.unwrap())
}

pub async fn put_dynamodb( 
    client: &DynamoClient,
    table_name: &str,
    query_params: Vec<(String, String)>
) -> Result<(), aws_sdk_dynamodb::Error> {

    let new_params = query_params.iter().map(|params| {
        (
            params.0.clone(), 
            AttributeValue::S(params.1.clone()),
        ) 
    }).collect::<Vec<_>>();

    let items_map: HashMap<String, AttributeValue> = HashMap::from_iter(new_params);

    client.put_item()
        .table_name(table_name)
        .set_item(Some(items_map)).send().await?;

    Ok(())
}

async fn on_connect(socket: SocketRef) {
    info!("socket connected: {}", socket.id);

    socket.on(
        "/",
        |socket: SocketRef, Data::<String>(room)| async move {
            info!("Received join: {:?}", room);
            let _ = socket.leave_all();
            let _ = socket.join(room.clone());
            let _ = socket.emit("messages", "hi");
        },
    );

    socket.on(
        "test",
        |socket: SocketRef| async move {
            info!("Received join!");
        }, 
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let addr = "[::1]:50051";
    let (layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);

    //find out how to make a global client
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