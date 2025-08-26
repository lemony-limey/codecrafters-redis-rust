use crate::{
    commands::redis_command::RedisCommand,
    parser::{
        redis_value,
        resp_error::RESPError,
        resp_value::RESPValue,
        set::parse_set_command,
    },
};
use anyhow::anyhow;
use bytes::Bytes;
use std::collections::VecDeque;


/// Parse Redis commands from a given input string.
///
/// Returns a tuple (x, y), where:
///     x: The remaining unparsed input
///     y: A VecDeque containing all parsed commands
pub fn redis_commands(input: Bytes) -> anyhow::Result<(String, Option<VecDeque<RedisCommand>>)>
{
    println!("input: {}", str::from_utf8(&input)?);

    // Everything should be contained within a RESP array
    let result = redis_value(str::from_utf8(&input)?);
    let Ok((remaining_input, command_value)) = result else {
        return Err(anyhow!(RESPError::UnknownStartingByte(
            format!(
                "redis_commands(): could not parse array of commands: {:#?}, due to {:#?}",
                input,
                result,
            )
        )));
    };
    let mut parsed_commands: VecDeque<RedisCommand> = VecDeque::new();

    // Commands are arrays of bulk strings
    let RESPValue::Array(mut resp_array) = command_value else {
        println!("Failed to parse whole command");
        return Err(anyhow!(RESPError::ParseError(
            format!("redis_commands(): dodgy array: {:#?}", command_value)
        )));
    };

    // Loop here and get as many commands as possible.
    // We are currently assuming here that command names are only one word.
    while let Some(front_of_queue) = resp_array.pop_front()
    {
        // A command name is always encoded as a bulk string
        let RESPValue::BulkString(command_name) = front_of_queue else {
            println!("Failed to get command name");
            return Err(anyhow!("redis_commands(): dodgy bulk string: {:#?}", front_of_queue));
        };

        // println!("command name: {}", command_name);

        // Attempt to match the bulk string to the name of a valid command
        let current_command: RedisCommand;
        match command_name.to_lowercase().as_str()
        {
            "echo" => {
                let front_of_queue = resp_array.pop_front();
                let Some(RESPValue::BulkString(message)) = front_of_queue else {
                    return Err(anyhow!(
                        "redis_commands(): echo: failed to parse argument {:#?}",
                        front_of_queue
                    ));
                };

                current_command = RedisCommand::Echo { message: message.clone() }
            },
            "get" => {
                // Key
                let front_of_queue = resp_array.pop_front();
                let Some(RESPValue::BulkString(key)) = front_of_queue else {
                    return Err(anyhow!(
                        "redis_commands(): get: failed to parse argument {:#?}",
                        front_of_queue
                    ));
                };

                current_command = RedisCommand::Get { key }
            },
            "lrange" => {
                // Key
                let front_of_queue = resp_array.pop_front();
                let Some(RESPValue::BulkString(key)) = front_of_queue else {
                    return Err(anyhow!(
                        "redis_commands(): get: failed to parse argument {:#?}",
                        front_of_queue
                    ));
                };

                // Start
                let front_of_queue = resp_array.pop_front();
                let Some(RESPValue::BulkString(start)) = front_of_queue else {
                    return Err(anyhow!(
                        "redis_commands(): get: failed to parse argument {:#?}",
                        front_of_queue
                    ));
                };

                // End
                let front_of_queue = resp_array.pop_front();
                let Some(RESPValue::BulkString(end)) = front_of_queue else {
                    return Err(anyhow!(
                        "redis_commands(): get: failed to parse argument {:#?}",
                        front_of_queue
                    ));
                };

                current_command = RedisCommand::LRange {
                    key,
                    start: start.parse::<i64>()?,
                    end: end.parse::<i64>()?,
                }
            }
            "ping" => {
                current_command = RedisCommand::Ping
            },
            "rpush" => {
                // Key
                let front_of_queue = resp_array.pop_front();
                let Some(RESPValue::BulkString(key)) = front_of_queue else {
                    return Err(anyhow!(
                        "redis_commands(): rpush: failed to parse argument {:#?}",
                        front_of_queue
                    ));
                };

                // Values
                let mut values = VecDeque::new();
                while let Some(value) = resp_array.pop_front()
                {
                    values.push_back(value);
                }

                current_command = RedisCommand::RPush { key, value: values }
            },
            "set" => {
                let RedisCommand::Set { key, value, px } = parse_set_command(&mut resp_array)? else {
                    return Err(anyhow!("redis_commands(): set: could not parse command"));
                };

                current_command = RedisCommand::Set { key, value, px }
            },
            _ => break,
        }

        // Append the new entry to the back of the command queue
        parsed_commands.push_back(current_command);
    }

    // If no commands were parsed, parsed_commands should be None as opposed to an empty VecDeque
    if parsed_commands.is_empty()
    {
        Ok((
            remaining_input.to_string(),
            None
        ))
    }
    else
    {
        Ok((
            remaining_input.to_string(),
            Some(parsed_commands),
        ))
    }
}
