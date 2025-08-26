use crate::parser::resp_value::RESPValue;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum RedisCommand
{
    Echo {
        message: String
    },
    Get {
        key: String,
    },
    Ping,
    RPush {
        key: String,
        value: RESPValue,
    },
    Set {
        key: String,
        value: RESPValue,
        px: Option<DateTime<Utc>>,
    },
}
