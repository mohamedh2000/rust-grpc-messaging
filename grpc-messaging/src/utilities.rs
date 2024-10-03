use aws_sdk_dynamodb::{
    types::{AttributeValue, KeysAndAttributes},
    Client as DynamoClient,
};
use std::collections::HashMap;
use tracing::info;
use socketioxide::extract::{Data, SocketRef};

use std::env;

mod dynamo_operations; 
pub mod chat_grpc;
pub mod socket_handler;