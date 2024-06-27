use tonic::{transport::Server, Request, Response, Status, Code};
use aws_sdk_dynamodb::{types::{AttributeValue, KeysAndAttributes}, Client};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use chat::chat_server::{Chat, ChatServer};
use chat::{User, UserRoomResponse, Room};
use std::collections::HashMap;
use std::convert::From;
use std::env::var;
use lazy_static::lazy_static;

pub mod chat { 
    tonic::include_proto!("chat");
}

#[derive(Default)]
pub struct ChatService {}

lazy_static! {
    static ref ROOM_TABLE: String = var("ROOM_TABLE").unwrap();
    static ref USER_TABLE: String = var("USER_TABLE").unwrap(); 
}

// pub struct DynamoDbDataBaseServer<T> 
// where 
//     T: 'static + DataStore + Send + Sync,
// {
//     data_store: Arc<T>,
// }

// impl<T> DynamoDbDataBaseServer<T>
//     where T: 'static + DataStore + Send + Sync,
//     {
//         pub fn new(store: T) -> DynamoDbDataBaseServer<T> {
//             DynamoDbDataBaseServer {
//                 data_store: Arc::new(store),
//             }
//         }
//     }


#[tonic::async_trait]
impl Chat for ChatService {

    async fn user_rooms(
        &self, 
        request: Request<User>,
    ) -> Result<Response<UserRoomResponse>, Status> { 
        println!("Got a request from {:?}", request.remote_addr());

        //find out how to make a global client
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);


        let user_id: String = request.into_inner().user_id;
        let results = query_dynamodb(&client, &*USER_TABLE, vec![(String::from("userId"), user_id)]).await.unwrap();
        let results = results.get(&*USER_TABLE).unwrap();

        let user_rooms = &results[0];
        let rooms_vec = AttributeValue::as_l(user_rooms.get("rooms").unwrap()).unwrap();
        let rooms_vec = rooms_vec.iter().map(|room| {
            (String::from("roomId"), String::from(AttributeValue::as_n(room).unwrap()))
        }).collect::<Vec<_>>();

        match UserRoomResponse::get_batch_results(rooms_vec, &*ROOM_TABLE, &client).await { 
            Ok(user_rooms) => Ok(Response::new(user_rooms)),
            Err(err) => Err(Status::new(Code::Aborted, err.to_string())),
        }
 
    }

    // async fn send_message_stream()
}

trait DynamoResultConversions<T, U> {
    fn convert_to_message(to_convert: &U) -> T;
}

impl DynamoDbQuery<UserRoomResponse> for UserRoomResponse {}

trait DynamoDbQuery<T> where T: DynamoResultConversions<T, Vec<HashMap<String, AttributeValue>>> {
    //makes a batch get request to dynamodb and then converts data to the passed in type T
    async fn get_batch_results( map: Vec<(String, String)>, table_name:&str, client: &Client) -> Result<T, aws_sdk_dynamodb::Error> {
        let result = query_dynamodb(&client, table_name, map).await?;
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
    client: &Client,
    table_name: &str,
    query_params: Vec<(String, String)>,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let chat_service = ChatService::default();

    println!("ChatServer listening on {}", addr);

    Server::builder()
        .add_service(ChatServer::new(chat_service))
        .serve(addr)
        .await?;

    Ok(())
}