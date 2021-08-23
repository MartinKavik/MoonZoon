use crate::*;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CorId(Ulid);

impl CorId {
    pub fn new() -> Self {
        CorId(Ulid::generate())
    }
}

impl fmt::Display for CorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CorId {
    type Err = DecodingError;

    fn from_str(cor_id: &str) -> Result<Self, Self::Err> {
        Ok(CorId(cor_id.parse()?))
    }
}

impl Serialize for CorId {
    fn serialize(&self) -> Result<Intermediate, local_serde::Error> {
        Ok(Intermediate::String(self.to_string()))
    }
}

impl Deserialize for CorId {
    fn deserialize(intermediate: &Intermediate) -> Result<Self, local_serde::Error> {
        intermediate
            .as_str()
            .ok_or_else(|| {
                local_serde::Error::invalid_value("CorId can be deserialized only from String")
            })?
            .parse()
            .map_err(|error| local_serde::Error::invalid_value(error))
    }
}
