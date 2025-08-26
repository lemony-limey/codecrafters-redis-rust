use crate::{parser::resp_value::RESPValue, write_response, Database};
use anyhow::anyhow;
use nom::AsBytes;
use std::collections::{LinkedList, VecDeque};


pub async fn rpush(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    database: Database,
    key: String,
    values: VecDeque<RESPValue>,
) -> anyhow::Result<()>
{
    // Check if the key is in the database, and add it if not.
    let mut database = database.lock().await;

    if !database.contains_key(&key)
    {
        let linked_list = LinkedList::new();
        database.insert(
            key.clone(),
            (RESPValue::List(linked_list), None),
        );
    }

    // If key does exist, ensure that it is associated with a List, not some other RESPValue.
    let Some((RESPValue::List(linked_list), _expiry)) = database.get_mut(&key) else {
        // TODO: Respond with failure

        return Err(anyhow!("rpush(): specified key is not associated with a linked list"));
    };

    // Append the key to the end of the linked list
    for value in values
    {
        linked_list.push_back(value);
    }
    let list_size = linked_list.len();

    // Respond with success
    let response = RESPValue::Integer(list_size as i64)
        .to_resp_encoded_string()
        .expect("could not create resp string for specified integer");
    
    write_response(writer, response.as_bytes()).await
}
