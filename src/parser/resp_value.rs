use crate::parser::redis_value;
use nom::{
    bytes::complete::take,
    character::complete::{char, crlf, digit1, not_line_ending},
    combinator::map_res,
    IResult,
    Parser,
};
use std::collections::{LinkedList, VecDeque};


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
    List(LinkedList<RESPValue>),
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
    pub fn to_resp_encoded_string(&self) -> Option<String>
    {
        match self
        {
            RESPValue::SimpleString(string) => {
                Some(format!("+{}\r\n", string))
            }
            RESPValue::SimpleError(error) => {
                Some(format!("-{}\r\n", error).into())
            }
            RESPValue::Integer(integer) => {
                Some(format!(":{}\r\n", integer))
            }
            RESPValue::BulkString(string) => {
                Some(format!("${}\r\n{}\r\n", string.len(), string))
            }
            RESPValue::Array(array) => {
                let mut string = format!("*{}\r\n", array.len());

                for value in array
                {
                    string.push_str(
                        value.to_resp_encoded_string()
                            .expect("could not encode value into resp string")
                            .as_str()
                    );
                }

                Some(string)
            }
            RESPValue::List(_) => unimplemented!(),
            _ => None,
        }
    }
}


/// Parse a simple string and return its contents.
/// Simple strings begin with a "+".
pub(crate) fn resp_simple_string(input: &str) -> IResult<&str, RESPValue>
{
    // Consume a '+'
    let (remaining_input, _parsed_char) = char('+')(input)?;
    // Get the string
    let (remaining_input, simple_string) = not_line_ending(remaining_input)?;
    // Consume the crlf
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    Ok((
        remaining_input,
        RESPValue::SimpleString(simple_string.to_owned()),
    ))
}


/// Parse a simple error string and return its contents.
///
/// Simple error strings begin with a "-".
pub(crate) fn resp_simple_error(input: &str) -> IResult<&str, RESPValue>
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


pub(crate) fn resp_integer(input: &str) -> IResult<&str, RESPValue>
{
    // Consume a ':'
    let (remaining_input, _parsed_char) = char(':')(input)?;

    // Get the integer value
    let (remaining_input, integer_value) = parse_i64(remaining_input)?;

    // Consume the crlf
    let (remaining_input, _parsed_crlf) = crlf(remaining_input)?;

    Ok((
        remaining_input,
        RESPValue::Integer(integer_value),
    ))
}


pub(crate) fn resp_bulk_string(input: &str) -> IResult<&str, RESPValue>
{
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

    Ok((
        remaining_input,
        RESPValue::BulkString(inner_string.to_owned()),
    ))
}


pub(crate) fn resp_array(input: &str) -> IResult<&str, RESPValue>
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
    // TODO: Testing
}
