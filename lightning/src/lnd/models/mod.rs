mod hodl_invoice;
mod info;
mod invoice;
mod invoice_request;
mod lnd_payment;
pub use hodl_invoice::*;
pub use info::*;
pub use invoice::*;
pub use invoice_request::*;
pub use lnd_payment::*;

use std::fmt::Display;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndResponse<T> {
    pub result: T,
}
impl<T> LndResponse<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    pub fn inner(&self) -> T {
        self.result.clone()
    }
}
impl<T> TryFrom<&String> for LndResponse<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    type Error = anyhow::Error;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}
impl<T> TryFrom<String> for LndResponse<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl<T> TryInto<String> for LndResponse<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl<T> Display for LndResponse<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndErrorDetail {
    code: i32,
    message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndError {
    error: LndErrorDetail,
}
impl TryFrom<&String> for LndError {
    type Error = anyhow::Error;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}
impl TryFrom<String> for LndError {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl Display for LndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
