// The following resources were immensely helpful when writing this parser:
// https://dpbriggs.ca/blog/Implementing-A-Copyless-Redis-Protocol-in-Rust-With-Parsing-Combinators
// https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md
// I have used some of the error handling code from "Implementing A Copyless Redis Protocol in
// Rust with Parsing Combinators" and refactored my RESPValues types to use some of their ones.

pub mod resp_error;
pub mod resp_value;
mod set;
pub(crate) mod redis_commands;

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
