use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

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