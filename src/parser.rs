// The following resources were immensely helpful when writing this parser:
// https://dpbriggs.ca/blog/Implementing-A-Copyless-Redis-Protocol-in-Rust-With-Parsing-Combinators
// https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md

use nom::{
    bytes::complete::take,
    character::complete::{
        crlf,
        digit1,
        not_line_ending,
    },
    combinator::peek,
    error::{
        Error,
        ErrorKind,
    },
    Err,
    IResult,
    Parser,
    character::complete::char,
    combinator::map_res,
};

#[derive(Default)]
pub struct RESPParser;

/// \r\n is the RESP terminator, which ALWAYS separates parts.
/// Simple strings will never contain a \r or \n, but bulk strings
/// could have these as binary data.
pub enum RESPValue
{                              // First character
    SimpleString(String),     // +
    SimpleError(String),      // -
    Integer(i64),              // :
    BulkString(String),       // $
    Array(Vec<RESPValue>), // *
                     //    so, and empty array is *0\r\n
    Null,            // _\r\n  -or-  *-1\r\n for a null array
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

pub enum RedisCommand
{
    Ping,
    Echo { message: String },
}

pub fn redis_command(input: String) -> anyhow::Result<(String, RedisCommand)>
{
    let Ok((remaining_input, command_value)) = resp_value(input.as_str()) else {
        return Err("".to_string());
    };
    let mut command: RedisCommand = RedisCommand::Ping;

    // Commands are arrays of bulk strings
    let RESPValue::Array(resp_array) = command_value else {
        return Err("".to_string());
    };

    // The first (and possibly second) bulk strings form the name of the command
    let RESPValue::BulkString(command_name) = &resp_array[0] else {
        return Err("".to_string());
    };

    match command_name.as_str()
    {
        "ping" => {
            command = RedisCommand::Ping;
        },
        "echo" => {
            let RESPValue::BulkString(message) = &resp_array[1] else {
                return Err("".to_string());
            };
            command = RedisCommand::Echo { message: message.clone() };
        },
        _ => return Err("".to_string()),
    }

    Ok((
        remaining_input.to_string(),
        command,
    ))
}

/// Checks the first non-whitespace character and attempts to parse an instance
/// of the RESP data type associated with that character (e.g., if the first
/// character of the input string is a '+', we know to look for a simple string).
pub fn resp_value(input: &str) -> IResult<&str, RESPValue>
{
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
        _ => Err(Err::Failure(Error { input: input.clone(), code: ErrorKind::Fail }))
    }
}

/// Parse a simple string and return its contents.
/// Simple strings begin with a "+".
fn resp_simple_string(input: &str) -> IResult<&str, RESPValue>
{
    // Consume a '+'
    let (remaining_input, _) = char('+')(input)?;
    // Get the string
    let (remaining_input, simple_string) = not_line_ending(remaining_input)?;
    // Consume the crlf
    let (remaining_input, _) = crlf(remaining_input)?;
    
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
    let (remaining_input, _) = char('-')(input)?;
    // Get the inner string
    let (remaining_input, simple_error) = not_line_ending(remaining_input)?;
    // Consume the crlf
    let (remaining_input, _) = crlf(remaining_input)?;

    Ok((
        remaining_input,
        RESPValue::SimpleError(simple_error.to_owned()),
    ))
}

fn resp_integer(input: &str) -> IResult<&str, RESPValue>
{
    // Consume a ':'
    let (remaining_input, _) = char(':')(input)?;

    // Get the integer value
    let (remaining_input, integer_value) = parse_i64(remaining_input)?;

    // Consume the crlf
    let (remaining_input, _) = crlf(remaining_input)?;

    Ok((
        remaining_input,
        RESPValue::Integer(integer_value),
    ))
}

fn resp_bulk_string(input: &str) -> IResult<&str, RESPValue>
{
    // Consume a '$'
    let (remaining_input, _) = char('$')(input)?;

    // Get the length of the bulk string
    let (remaining_input, string_length) = parse_u32(remaining_input)?;

    // Consume the first CRLF delimiter
    let (remaining_input, _) = crlf(remaining_input)?;

    // Get the string
    let (remaining_input, inner_string) = take(string_length)(remaining_input)?;

    // Consume the second CRLF delimiter
    let (remaining_input, _) = crlf(remaining_input)?;

    Ok((
        remaining_input,
        RESPValue::BulkString(inner_string.to_owned()),
    ))
}

fn resp_array(input: &str) -> IResult<&str, RESPValue>
{
    // Parse the *
    let (remaining_input, _) = char('*')(input)?;

    // Get the length of the array
    let (remaining_input, array_length) = parse_u32(remaining_input)?;

    // Consume the first CRLF delimiter
    let (remaining_input, _) = crlf(remaining_input)?;

    // Get the contents of the array
    let mut array_contents: Vec<RESPValue> = Vec::with_capacity(array_length as usize);

    let mut remaining_input = remaining_input;
    for i in 0..array_length
    {
        let (current_remaining_input, next_entry) = resp_value(remaining_input)?;
        remaining_input = current_remaining_input;

        array_contents.push(next_entry);
    }

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

}
