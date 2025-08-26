use std::collections::VecDeque;

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
    pub fn to_resp_encoded_string(&self) -> Option<String>
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
