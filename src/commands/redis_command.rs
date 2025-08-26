use crate::parser::resp_value::RESPValue;
use chrono::{DateTime, Utc};

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
