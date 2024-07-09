use super:: {
    DynamoClient,
    AttributeValue,
    HashMap,
    env
};

use tonic::{
    Request, 
    Response, 
    Status, 
    Code
};

use crate::utilities::dynamo_operations::query_dynamodb;
use chat::{
    chat_server::Chat, 
    User, 
    UserRoomResponse, 
    Room, 
    ChatMessage, 
    Empty, 
    RoomRequest, 
    FriendRequest, 
    FriendResponse
};

pub mod chat { 
    tonic::include_proto!("chat");
}

pub struct ChatService {
    dynamodb_client: DynamoClient,
}

impl ChatService {
    pub fn new(dynamodb_client : DynamoClient) -> Self {
        ChatService{
            dynamodb_client, 
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

    //can only send a request if you haven't sent already 
    async fn send_request_friend(
        &self, 
        request: Request<FriendRequest>,
    ) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty{}))
    }

    async fn respond_friend_request (
        &self, 
        request: Request<FriendResponse>,
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

    // //TODO: Might not be needed
    // async fn send_message(
    //     &self, 
    //     request: Request<ChatMessage>,
    // ) -> Result<Response<Empty>, Status> {
    //     let chat_message = request.into_inner();
    //     let room_id = chat_message.room_id.clone();
    //     let message_table_env = &env::var("MESSAGE_TABLE").unwrap();
        
    //     let results = put_dynamodb(&self.dynamodb_client, message_table_env, 
    //         vec![
    //         (String::from("userId"), chat_message.user_id.clone()),
    //         (String::from("roomId"), chat_message.room_id.clone()),
    //         (String::from("username"), chat_message.username.clone()),
    //         (String::from("message"), chat_message.message.clone()),
    //         ]
    //     ).await;

    //     match results {
    //         Ok(_) => {
    //             let room_map = self.rooms.lock().unwrap();  
    //             let room_channel = room_map.get(&room_id).unwrap(); 
    //             room_channel.producer.send(chat_message);
    //             Ok(Response::new(Empty{}))
    //         },
    //         Err(e) => Err(Status::new(Code::Aborted, e.to_string())),
    //     }
    // }

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