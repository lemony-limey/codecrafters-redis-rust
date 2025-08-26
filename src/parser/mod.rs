// The following resources were immensely helpful when writing this parser:
// https://dpbriggs.ca/blog/Implementing-A-Copyless-Redis-Protocol-in-Rust-With-Parsing-Combinators
// https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md
// I have used some of the error handling code from "Implementing A Copyless Redis Protocol in
// Rust with Parsing Combinators" and refactored my RESPValues types to use some of their ones.

pub mod resp_error;
pub mod resp_value;
mod set;

use crate::{
    commands::redis_command::RedisCommand,
    parser::{
        resp_error::RESPError,
        resp_value::{
            resp_array,
            resp_bulk_string,
            resp_integer,
            resp_simple_error,
            resp_simple_string,
            RESPValue,
        },
        set::parse_set_command,
    },
};
use anyhow::anyhow;
use bytes::Bytes;
use nom::{
    bytes::complete::take,
    combinator::peek,
    error::ErrorKind,
    Err,
    IResult,
    Parser,
};
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

        println!("command name: {}", command_name);

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

                // Value
                let front_of_queue = resp_array.pop_front();
                let Some(value) = front_of_queue else {
                    return Err(anyhow!(
                        "redis_commands(): rpush: failed to parse argument {:#?}",
                        front_of_queue
                    ));
                };

                current_command = RedisCommand::RPush { key, value }
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


/// Checks the first non-whitespace character and attempts to parse an instance
/// of the RESP data type associated with that character (e.g., if the first
/// character of the input string is a '+', we know to look for a simple string).
pub fn redis_value(input: &str) -> IResult<&str, RESPValue>
{
    println!("redis_value start: {}", input);

    // Get the first character of the current string
    let (remaining_input, data_type) = peek(take(1usize)).parse(input)?;

    // Match the first character to determine what kind of data type
    // is to be parsed.
    match data_type
    {
        "+" => resp_simple_string(remaining_input),
        "-" => resp_simple_error(remaining_input),
        ":" => resp_integer(remaining_input),
        "$" => resp_bulk_string(remaining_input),
        "*" => resp_array(remaining_input),
        // "_" => {},
        // "#" => {},
        // "," => {},
        // "(" => {},
        // "!" => {},
        // "=" => {},
        // "%" => {},
        // "|" => {},
        // "~" => {},
        // ">" => {},
        // Indicate that parsing could not occur because first character of the string
        // was not recognised.
        _ => Err(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail }))
    }
}
