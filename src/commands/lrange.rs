use crate::parser::resp_value::RESPValue;
use crate::{write_response, Database};

const EMPTY_ARRAY: &[u8] = b"*0\r\n";


pub async fn lrange(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    database: Database,
    key: String,
    mut start_inclusive: i64,
    mut end_inclusive: i64,
) -> anyhow::Result<()>
{
    let database = database.lock().await;

    let mut response: String;

    // Get the linked list from the database
    let Some((RESPValue::List(linked_list), _expiry)) = database.get(&key) else {
        return write_response(writer, EMPTY_ARRAY).await;
    };

    let list_len = linked_list.len() as i64;
    if start_inclusive < -list_len
    {
        start_inclusive = 0;
    }
    else if start_inclusive < 0
    {
        start_inclusive += list_len;
    }

    if end_inclusive > list_len
    {
        end_inclusive = list_len - 1;
    }

    // Either of these conditions mean we must return an empty array.
    if start_inclusive >= list_len || end_inclusive < -list_len
    {
        return write_response(writer, EMPTY_ARRAY).await;
    }

    if end_inclusive < 0
    {
        end_inclusive += list_len;
    }

    response = format!("*{}\r\n", end_inclusive - start_inclusive + 1);
    println!("{:#?}", *linked_list);
    println!("start: {}, end: {}, list_len: {}", start_inclusive, end_inclusive, list_len);

    let response = linked_list.iter().clone()
        .skip(start_inclusive as usize)
        .take((end_inclusive - start_inclusive + 1) as usize)
        .fold(&mut response, |acc, next_value| {
            acc.push_str(
                next_value.to_resp_encoded_string()
                    .expect("could not encode resp value")
                    .as_str()
            );
            acc
        });

    println!("list_len afterwards: {}", linked_list.len());

    write_response(writer, response.as_bytes()).await
}