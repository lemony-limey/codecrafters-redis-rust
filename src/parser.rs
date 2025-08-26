// The following resources were immensely helpful when writing this parser:
// https://dpbriggs.ca/blog/Implementing-A-Copyless-Redis-Protocol-in-Rust-With-Parsing-Combinators
// https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md
// I have used some of the error handling code from "Implementing A Copyless Redis Protocol in
// Rust with Parsing Combinators" and refactored my RESPValues types to use some of their ones.

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::Formatter;
use bytes::{Bytes, BytesMut};
use nom::{
    bytes::complete::take,
    character::complete::{
        crlf,
        digit1,
        not_line_ending,
    },
    combinator::peek,
    error::{
        ErrorKind,
    },
    Err,
    IResult,
    Parser,
    character::complete::char,
    combinator::map_res,
};
use tokio_util::codec::Decoder;

#[derive(Default)]
pub struct RESPParser;

/// \r\n is the RESP terminator, which ALWAYS separates parts.
/// Simple strings will never contain a \r or \n, but bulk strings
/// could have these as binary data.
#[derive(Debug, Clone)]
pub enum RESPValue
{                              // First character
    SimpleString(String),     // +
    SimpleError(String),      // -
    Integer(i64),              // :
    BulkString(String),       // $
    Array(VecDeque<RESPValue>), // *
    //    so, and empty array is *0\r\n
    Null,            // _\r\n  -or-  *-1\r\n for a null array
    NullBulkString,  // $-1\r\n
    //    so, and empty array is *0\r\n
    Boolean,         // #  #<t|f>\r\n
    Double,          // ,
    BigNumber,       // (
    BulkError,       // !
    VerbatimString,  // =
    Map,             // %
    Attribute,       // |
    Set,             // ~
    Push,            // >
}

impl RESPValue
{
    pub fn encode(&self) -> Option<String>
    {
        match self
        {
            RESPValue::SimpleString(string) => {
                format!("+{}\r\n", string).into()
            }
            RESPValue::SimpleError(error) => {
                format!("-{}\r\n", error).into()
            }
            RESPValue::Integer(integer) => {
                format!(":{}\r\n", integer).into()
            }
            RESPValue::BulkString(string) => {
                format!("${}\r\n{}\r\n", string.len(), string).into()
            }
            _ => None
        }
    }
}


#[derive(thiserror::Error)]
pub enum RESPError
{
    #[error("Unexpected end to the data stream occurred")]
    UnexpectedEnd,
    #[error("Unknown starting byte")]
    UnknownStartingByte(String),
    #[error("An I/O error occurred")]
    IOError(std::io::Error),
    #[error("Failed to parse an integer")]
    IntParseFailure,
    #[error("Bad bulk string size")]
    BadBulkStringSize(i64),
    #[error("Bad array size")]
    BadArraySize(i64),
    #[error("Failed to parse input")]
    ParseError(String),
}

impl From<std::io::Error> for RESPError
{
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl std::fmt::Debug for RESPError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        writeln!(f, "{self}")?;
        if let Some(e) = self.source()
        {
            writeln!(f, "\tCaused by: {e:?}")?;
        }
        match self
        {
            RESPError::UnknownStartingByte(error_string)
                | RESPError::ParseError(error_string) => writeln!(f, "{}", error_string)?,
            _ => {}
        }
        Ok(())
    }
}

// TODO: Implement tokio Decode
// impl Decoder for RESPParser
// {
//     type Item = RESPValue;
//     type Error = RESPError;
//
//     fn decode(&mut self, _src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>
//     {
//         todo!()
//     }
// }

#[derive(Debug, Clone)]
pub enum RedisCommand
{
    Ping,
    Echo { message: String },
    Set { key: String, value: RESPValue, },
    Get { key: String, },
}

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
        return Err(RESPError::UnknownStartingByte(
            format!(
                "redis_commands(): could not parse array of commands: {:#?}, due to {:#?}",
                input,
                result,
            )
        ).into());
    };
    let mut parsed_commands: VecDeque<RedisCommand> = VecDeque::new();

    // Commands are arrays of bulk strings
    let RESPValue::Array(mut resp_array) = command_value else {
        println!("Failed to parse whole command");
        return Err(RESPError::ParseError(
            format!("redis_commands(): dodgy array: {:#?}", command_value)
        ).into());
    };

    // Loop here and get as many commands as possible.
    // We are currently assuming here that command names are only one word.
    while let Some(front_of_queue) = resp_array.pop_front()
    {
        // A command name is always encoded as a bulk string
        let RESPValue::BulkString(command_name) = front_of_queue else {
            println!("Failed to get command name");
            return Err(
                RESPError::ParseError(
                    format!("redis_commands(): dodgy bulk string: {:#?}", front_of_queue)
                ).into()
            );
        };

        println!("command name: {}", command_name);

        // Attempt to match the bulk string to the name of a valid command
        let current_command: RedisCommand;
        match command_name.to_lowercase().as_str()
        {
            "ping" => {
                current_command = RedisCommand::Ping;
            },
            "echo" => {
                let front_of_queue = resp_array.pop_front();
                let Some(RESPValue::BulkString(message)) = front_of_queue else {
                    return Err(RESPError::ParseError(
                        format!(
                            "redis_commands(): echo: failed to parse argument {:#?}",
                            front_of_queue
                        )
                    ).into());
                };
                current_command = RedisCommand::Echo { message: message.clone() };
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

/// Parse a simple string and return its contents.
/// Simple strings begin with a "+".
fn resp_simple_string(input: &str) -> IResult<&str, RESPValue>
{
    println!("resp_string: {}", input);
    // Consume a '+'
    let (remaining_input, _parsed_char) = char('+')(input)?;
    // Get the string
    let (remaining_input, simple_string) = not_line_ending(remaining_input)?;
    // Consume the crlf
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    println!("resp string result: ({}, {})", remaining_input, simple_string);

    Ok((
        remaining_input,
        RESPValue::SimpleString(simple_string.to_owned()),
    ))
}

/// Parse a simple error string and return its contents.
///
/// Simple error strings begin with a "-".
fn resp_simple_error(input: &str) -> IResult<&str, RESPValue>
{
    // Consume a '-'
    let (remaining_input, _parsed_char) = char('-')(input)?;
    // Get the inner string
    let (remaining_input, simple_error) = not_line_ending(remaining_input)?;
    // Consume the crlf
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    Ok((
        remaining_input,
        RESPValue::SimpleError(simple_error.to_owned()),
    ))
}

fn resp_integer(input: &str) -> IResult<&str, RESPValue>
{
    println!("resp_integer: {}", input);
    // Consume a ':'
    let (remaining_input, _parsed_char) = char(':')(input)?;

    // Get the integer value
    let (remaining_input, integer_value) = parse_i64(remaining_input)?;

    // Consume the crlf
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    println!("resp int result: ({}, {})", remaining_input, integer_value);

    Ok((
        remaining_input,
        RESPValue::Integer(integer_value),
    ))
}

fn resp_bulk_string(input: &str) -> IResult<&str, RESPValue>
{
    // println!("bulk_string: {}", input);
    // Consume a '$'
    let (remaining_input, _parsed_char) = char('$')(input)?;

    // Get the length of the bulk string
    let (remaining_input, string_length) = parse_u32(remaining_input)?;

    // Consume the first CRLF delimiter
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    // Get the string
    let (remaining_input, inner_string) = take(string_length)(remaining_input)?;

    // Consume the second CRLF delimiter
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    println!("bulk string result: ({}, {})", remaining_input, inner_string);

    Ok((
        remaining_input,
        RESPValue::BulkString(inner_string.to_owned()),
    ))
}

fn resp_array(input: &str) -> IResult<&str, RESPValue>
{
    // println!("resp_array: {}", input);

    // Parse the *
    let (remaining_input, _parsed_char) = char('*')(input)?;

    // Get the length of the array
    let (remaining_input, array_length) = parse_u32(remaining_input)?;

    // Consume the first CRLF delimiter
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    // Get the contents of the array
    let mut array_contents: VecDeque<RESPValue> = VecDeque::with_capacity(array_length as usize);

    let mut remaining_input = remaining_input;
    for _ in 0..array_length
    {
        let (current_remaining_input, next_entry) = redis_value(remaining_input)?;
        remaining_input = current_remaining_input;

        array_contents.push_back(next_entry);
    }

    println!("resp array result: ({}, {:#?})", remaining_input, array_contents);

    Ok((
        remaining_input,
        RESPValue::Array(array_contents),
    ))
}

fn parse_i64(input: &str) -> IResult<&str, i64>
{
    map_res(digit1, str::parse::<i64>).parse(input)
}

fn parse_u32(input: &str) -> IResult<&str, u32>
{
    map_res(digit1, str::parse::<u32>).parse(input)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nom::bytes::complete::take;

    // TODO: Testing
}
