use std::collections::VecDeque;
use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use nom::Parser;
use crate::commands::redis_command::RedisCommand;
use crate::parser::resp_value::RESPValue;


pub fn parse_set_command(resp_array: &mut VecDeque<RESPValue>) -> anyhow::Result<RedisCommand>
{
    // Key
    let front_of_queue = resp_array.pop_front();
    let Some(RESPValue::BulkString(key)) = front_of_queue else {
        return Err(anyhow!(
            "redis_commands(): set: failed to parse argument {:#?}",
            front_of_queue
        ));
    };

    // Value
    let front_of_queue = resp_array.pop_front();
    let Some(value) = front_of_queue else {
        return Err(anyhow!(
            "redis_commands(): set: failed to parse argument {:#?}",
            front_of_queue
        ));
    };

    // Optional arguments
    let mut px: Option<DateTime<Utc>> = None;
    match resp_array.front() {
        Some(RESPValue::BulkString(string)) => {
            if string.eq_ignore_ascii_case("px")
            {
                let _px_string = resp_array.pop_front();
                let px_value_string = resp_array.pop_front();
                let Some(RESPValue::BulkString(px_value)) = px_value_string else {
                    return Err(anyhow!(
                        "redis_commands(): set: failed to parse argument {:#?}",
                        px_value_string
                    ));
                };
                px = Some(
                    Utc::now()
                    .checked_add_signed(Duration::milliseconds(px_value.parse()?))
                    .expect("could not add px to current time")
                );
            }
        },
        _ => {}
    }

    Ok(RedisCommand::Set { key, value, px })
}