use nom::Err;
use nom::bytes::complete::take;
use nom::{
    IResult,
    Parser,
    bytes::complete::{tag, take_while_m_n},
    character::complete::char,
    combinator::map_res,
};
use nom::bytes::take_while;
use nom::character::complete::{crlf, digit1, not_line_ending, space0};
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::number::complete::be_i64;

/// \r\n is the RESP terminator, which ALWAYS separates parts.
/// Simple strings will never contain a \r or \n, but bulk strings
/// could have these as binary data.
enum RESPValue
{                         // First character, format example
    SimpleString(String), // +  e.g. +OK\r\n
    SimpleError(String),  // -  e.g. -Error message\r\n
    Integer(i64),         // :  [<+|->]<value>\r\n,
    BulkString(String),   // $  $<length>\r\n<data>\r\n
                     //    or  $-1\r\n for null bulk string
    Array(Vec<RESPValue>),  // *  *<no-of-elems>\r\n<elem-1>...<elem-n-1><elem-n>
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

/// Checks the first non-whitespace character and attempts to parse an instance
/// of the RESP data type associated with that character (e.g., if the first
/// character of the input string is a '+', we know to look for a simple string).
fn resp_command(input: &str) -> IResult<&str, &str>
{
    // Get the first character of the current string
    let (input_remaining, data_type) = take(1usize)(input)?;

    // Match the first character to determine what kind of data type
    // is to be parsed.
    match data_type
    {
        "+" => {},
        "-" => {},
        ":" => {},
        "$" => {},
        "*" => {},
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
        _ => {}
    }

    Ok(("", ""))
}

fn resp_value(input: &str) -> IResult<&str, RESPValue>
{
    
}

/// Parse a simple string and return its contents.
/// Simple strings begin with a "+".
fn simple_string(input: &str) -> IResult<&str, &str>
{
    // Consume a '+'
    let (remaining_input, _) = char('+')(input)?;
    // Get the string
    let (remaining_input, simple_string) = not_line_ending(remaining_input)?;
    // Consume the crlf
    let (remaining_input, _) = crlf(remaining_input)?;
    
    Ok((remaining_input, simple_string))
}

/// Parse a simple error string and return its contents.
///
/// Simple error strings begin with a "-".
fn simple_error(input: &str) -> IResult<&str, &str>
{
    // Consume a '-'
    let (remaining_input, _) = char('-')(input)?;
    // Get the inner string
    let (remaining_input, simple_error) = not_line_ending(remaining_input)?;
    // Consume the crlf
    let (remaining_input, _) = crlf(remaining_input)?;

    Ok((remaining_input, simple_error))
}

fn resp_integer(input: &str) -> IResult<&str, i64>
{
    // Consume a ':'
    let (remaining_input, _) = char(':')(input)?;

    // Get the integer value
    let (remaining_input, integer_value) = parse_i64(remaining_input)?;

    // Consume the crlf
    let (remaining_input, _) = crlf(remaining_input)?;

    Ok((remaining_input, integer_value))
}

fn bulk_string(input: &str) -> IResult<&str, &str>
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

    Ok((remaining_input, ""))
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

    #[test]
    fn test_get_first_character()
    {
        let input = "+hello\r\n";
        let Ok((input_remaining, data_type)) = take(1usize)(input) else {
            panic!()
        };
    }
}
