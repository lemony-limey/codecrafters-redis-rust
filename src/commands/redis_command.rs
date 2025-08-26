use crate::parser::resp_value::RESPValue;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

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
        value: VecDeque<RESPValue>,
    },
    Set {
        key: String,
        value: RESPValue,
        px: Option<DateTime<Utc>>,
    },
}
