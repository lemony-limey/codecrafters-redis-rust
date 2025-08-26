use std::{
    error::Error,
    fmt::Formatter,
};

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