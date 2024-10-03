use super::{
    DynamoClient,
    AttributeValue,
    HashMap,
    KeysAndAttributes
};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{ 
    error::SdkError, operation::put_item::PutItemError, Error
};
use chrono::Utc;
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn prepare_dynamo_params(map: &HashMap<String, AttributeValue>, param_name: &str, primary_key: &str) -> Vec<(String, String)> {
    let param_attr = map.get(param_name).unwrap();
    let param_vec = AttributeValue::as_l(param_attr).unwrap();
    param_vec.iter().map(|attr| {
        (String::from(primary_key), String::from(AttributeValue::as_s(attr).unwrap()))
    }).collect::<Vec<_>>()
}

pub async fn put_dynamodb( 
    client: DynamoClient,
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

pub async fn query_dynamodb( //with the result make sure to get table_name and then you can iterate over the values
    client: &DynamoClient,
    table_name: &str,
    query_params: &Vec<(String, String)>
) -> Result<HashMap<String, Vec<HashMap<String, AttributeValue>>>, aws_sdk_dynamodb::Error> {

    let mut vector_attributes: Vec<HashMap<String, AttributeValue>> = vec![];
    for params in query_params {
        vector_attributes.push(
            HashMap::from([(
                params.0.clone(), 
                AttributeValue::S(params.1.clone()),
            )])
        )
    }; 

    let dynamo_query = KeysAndAttributes::builder().set_keys(Some(vector_attributes)).build().unwrap();
    let result = client.batch_get_item().request_items(table_name, dynamo_query).send().await?;

    Ok(result.responses.unwrap())
}

//TODO: Change name, this puts if the entry doesn't exist 
pub async fn query_dynamodb_singular(
    client: &DynamoClient,
    table_name: &str,
    query_params: &Vec<(String, String)>,
) -> Result<bool, PutItemError> {
    let mut attr_val_map = HashMap::new();

    for params in query_params {
        attr_val_map.insert(params.0.clone(), AttributeValue::S(params.1.clone()));
    }
    //TODO: COME BACK HERE AND CLEAN UP THE HASHING/SORTING
    let mut id_vec = vec![
        AttributeValue::as_s(attr_val_map.get("senderId").unwrap()).unwrap().clone(),
        AttributeValue::as_s(attr_val_map.get("receiverId").unwrap()).unwrap().clone()
    ];

    id_vec.sort();

    let mut s = DefaultHasher::new();
    id_vec.hash(&mut s);
    let id_hash = s.finish();

    println!("{:?}", attr_val_map);
    attr_val_map.insert(String::from("id"), AttributeValue::S(id_hash.to_string()));



    let result = client.put_item()
        .table_name(table_name)
        .set_item(Some(attr_val_map))
        .set_condition_expression(Some(format!("attribute_not_exists(id)")))
        .return_consumed_capacity(aws_sdk_dynamodb::types::ReturnConsumedCapacity::Total)
        .send()
        .await;

    match result {
        Ok(res) => {
            let consumed_cap = res.consumed_capacity.unwrap();
            Ok(consumed_cap.write_capacity_units().is_some())
        }
        Err(e) => {
            let err = e.into_service_error();
            if PutItemError::is_conditional_check_failed_exception(&err) {
                println!("{:?}", err);
                Ok(false)
            } else {
                Err(err)
            }
        }
    }

} 

pub async fn build_dynamo_client() -> DynamoClient {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    DynamoClient::new(&config)
}