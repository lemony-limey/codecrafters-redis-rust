use chrono::{DateTime, Utc};
use crate::parser::resp_value::RESPValue;

#[derive(Debug, Clone)]
pub enum RedisCommand
{
    Ping,
    Echo {
        message: String
    },
    Set {
        key: String,
        value: RESPValue,
        px: Option<DateTime<Utc>>,
    },
    Get {
        key: String,
    },
}
