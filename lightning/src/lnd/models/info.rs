use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInfo {
    identity_pubkey: String,
    block_height: u32,
}
impl TryFrom<&String> for LndInfo {
    type Error = anyhow::Error;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}
impl TryFrom<String> for LndInfo {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndInfo {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
