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
use crate::utilities::dynamo_operations::{query_dynamodb, build_dynamo_client};
use chat::{
    chat_server::Chat, 
    User, 
    Friend,
    UserInfoResponse,
    UserListResponse, 
    UserPfpRequest,
    Room, 
    Empty, 
    RoomRequest, 
    FriendRequest, 
    FriendResponse
};

pub mod chat { 
    tonic::include_proto!("chat");
}

pub struct ChatService {
    pub dynamodb_client: DynamoClient,
}

impl ChatService {
    pub fn new(dynamodb_client : DynamoClient) -> Self {
        ChatService{
            dynamodb_client, 
        }
    }
}

pub async fn create_service() -> ChatService {
    let dynamodb_client = build_dynamo_client().await;
    ChatService::new(dynamodb_client)
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

    async fn set_user_pfp(
        &self, 
        request: Request<UserPfpRequest>,
    ) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty{}))
    }

    async fn user_info(
        &self, 
        request: Request<User>,
    ) -> Result<Response<UserInfoResponse>, Status> { 
        println!("Got a request from {:?}", request.remote_addr());

        let user_table_env = &env::var("USER_TABLE").unwrap();

        let user_id: String = request.into_inner().user_id;
        let results = query_dynamodb(
            &self.dynamodb_client, 
            user_table_env, 
            &vec![(String::from("userId"), user_id)]).await.unwrap();
        let results = results.get(user_table_env).unwrap();

        let user_info = &results[0];

        //get username and profile pic
        let user_name = String::from(AttributeValue::as_s(user_info.get("userName").unwrap()).unwrap());
        let user_pfp = String::from(AttributeValue::as_s(user_info.get("profilePic").unwrap()).unwrap());
        
        //get list of rooms 
        let user_rooms = user_info.get("rooms").unwrap();
        let rooms_vec = AttributeValue::as_l(user_rooms).unwrap();
        let rooms_vec = rooms_vec.iter().map(|room| {
            (String::from("roomId"), String::from(AttributeValue::as_n(room).unwrap()))
        }).collect::<Vec<_>>();

        //get list of friends
        let user_friends = user_info.get("friends").unwrap();
        let friends_vec = AttributeValue::as_l(user_friends).unwrap();
        let friends_vec = friends_vec.iter().map(|friend| {
            (String::from("userId"), String::from(AttributeValue::as_s(friend).unwrap()))
        }).collect::<Vec<_>>();

        let mut query_param_map = HashMap::new();

        query_param_map.insert(String::from("room"), rooms_vec);
        query_param_map.insert(String::from("user"), friends_vec);

        match UserListResponse::get_batch_results(query_param_map, &self.dynamodb_client).await { 
            Ok(user_data) => {
                let data = Some(user_data);
                Ok(Response::new( 
                        UserInfoResponse {
                            user_name,
                            user_pfp,
                            user_data: data
                        }
                    )
                )
            },
            Err(err) => Err(Status::new(Code::Aborted, err.to_string())),
        }
    }

}

trait DynamoResultConversions<T, U> {
    fn convert_to_message(to_convert: &U) -> T;
}

impl DynamoDbQuery<UserListResponse> for UserListResponse {}

trait DynamoDbQuery<T> where T: DynamoResultConversions<T, HashMap<String, Vec<HashMap<String, AttributeValue>>>> {
    //makes a batch get request to dynamodb and then converts data to the passed in type T
    //map keys need to match .env table parameters
    async fn get_batch_results( map: HashMap<String, Vec<(String, String)>>, client: &DynamoClient) 
        -> Result<T, aws_sdk_dynamodb::Error> {

        let mut ret_map = HashMap::new();

        for key in map.keys() {
            let table_name = &env::var(format!("{}_TABLE", key.to_uppercase())).unwrap();
            let query_params = map.get(key).unwrap(); 
            let result = query_dynamodb(client, table_name, query_params).await?;
            let data = result.get(table_name).unwrap();
            ret_map.insert(key.clone(), data.clone()); //TODO: COME BACK TO THIS CLONE
        }

        Ok(T::convert_to_message(&ret_map))
    }
}

impl DynamoResultConversions<UserListResponse, HashMap<String, Vec<HashMap<String, AttributeValue>>>> for UserListResponse {
    fn convert_to_message(to_convert: &HashMap<String, Vec<HashMap<String, AttributeValue>>>) -> Self {

        let room_key = to_convert.get("room").unwrap();
        let friend_key = to_convert.get("user").unwrap();

        let rooms_metadata = room_key.iter().map(|room| {
            Room::convert_to_message(room)
        }).collect::<Vec<_>>();

        let friends_metadata =friend_key.iter().map(|friend| {
            Friend::convert_to_message(friend)
        }).collect::<Vec<_>>(); 

        UserListResponse {
            friends: friends_metadata,
            rooms: rooms_metadata
        }
    }
}

impl DynamoResultConversions<Room, HashMap<String, AttributeValue>> for Room {
    fn convert_to_message(to_convert: &HashMap<String, AttributeValue>) -> Self {
        Room {
            room_id: AttributeValue::as_s(to_convert.get("roomId").unwrap()).unwrap().parse::<u64>().unwrap(),
            room_name: String::from(AttributeValue::as_s(to_convert.get("roomName").unwrap()).unwrap()),
            room_pfp: String::from(AttributeValue::as_s(to_convert.get("roomPfp").unwrap()).unwrap()), 
        }
    }
}

impl DynamoResultConversions<Friend, HashMap<String, AttributeValue>> for Friend {
    fn convert_to_message(to_convert: &HashMap<String, AttributeValue>) -> Self {
        Friend {
            user_id: String::from(AttributeValue::as_s(to_convert.get("userId").unwrap()).unwrap()),
            user_name: String::from(AttributeValue::as_s(to_convert.get("userName").unwrap()).unwrap()),
            user_pfp: String::from(AttributeValue::as_s(to_convert.get("userPfp").unwrap()).unwrap()), 
        }
    }
}