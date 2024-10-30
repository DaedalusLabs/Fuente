use base64::prelude::*;
use lightning_invoice::Bolt11Invoice;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInvoiceRequest {
    form: String,
}
impl LndInvoiceRequest {
    pub fn new(amount: u64) -> Self {
        let body = LndInvoiceRequestBody {
            value: amount.to_string(),
        };
        Self {
            form: body.to_string(),
        }
    }
}
impl ToString for LndInvoiceRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInvoiceRequestBody {
    value: String,
}
impl ToString for LndInvoiceRequestBody {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl LndInvoiceRequestBody {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndInvoice {
    r_hash: String,
    payment_request: String,
    add_index: String,
    payment_addr: String,
}
impl TryFrom<String> for LndInvoice {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndInvoice {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndInvoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl LndInvoice {
    pub fn r_hash(&self) -> String {
        self.r_hash.clone()
    }
    pub fn r_hash_url_safe(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let url_safe = BASE64_URL_SAFE.encode(unsafe_str);
        url_safe
    }
    pub fn r_hash_hex(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let hex = hex::encode(unsafe_str);
        hex
    }
    pub fn payment_hash(&self) -> Vec<u8> {
        BASE64_STANDARD.decode(&self.payment_addr).unwrap()
    }
    pub fn payment_request(&self) -> String {
        self.payment_request.clone()
    }
    pub fn add_index(&self) -> String {
        self.add_index.clone()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndHodlInvoice {
    payment_addr: String,
    payment_request: String,
    add_index: String,
}
impl LndHodlInvoice {
    pub fn payment_hash(&self) -> Vec<u8> {
        self.payment_addr.as_bytes().to_vec()
    }
    pub fn payment_request(&self) -> String {
        self.payment_request.clone()
    }
    pub fn r_hash_url_safe(&self) -> anyhow::Result<String> {
        // let r_hash = self
        //     .payment_request
        //     .parse::<Bolt11Invoice>()
        //     .map_err(|e| anyhow::anyhow!(e.to_string()))?
        //     .p();
        let url_safe = BASE64_URL_SAFE.encode(self.payment_addr.as_bytes());
        Ok(url_safe)
    }
}
impl TryFrom<String> for LndHodlInvoice {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndHodlInvoice {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndHodlInvoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndSubscribeInvoice {
    pub r_hash: String,
}
impl TryInto<String> for LndSubscribeInvoice {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndHodlInvoiceState {
    settled: bool,
    state: HodlState,
    r_hash: String,
    payment_request: String,
}
impl TryFrom<String> for LndHodlInvoiceState {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndHodlInvoiceState {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndHodlInvoiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl LndHodlInvoiceState {
    pub fn settled(&self) -> bool {
        self.settled
    }
    pub fn state(&self) -> HodlState {
        self.state.clone()
    }
    pub fn r_hash(&self) -> String {
        self.r_hash.clone()
    }
    pub fn r_hash_url_safe(&self) -> String {
        let url_safe = BASE64_URL_SAFE.encode(self.r_hash.as_bytes());
        url_safe
    }
    pub fn payment_request(&self) -> String {
        self.payment_request.clone()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HodlState {
    OPEN,
    ACCEPTED,
    CANCELED,
    SETTLED,
}
impl TryFrom<String> for HodlState {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "OPEN" => Ok(HodlState::OPEN),
            "ACCEPTED" => Ok(HodlState::ACCEPTED),
            "CANCELED" => Ok(HodlState::CANCELED),
            "SETTLED" => Ok(HodlState::SETTLED),
            _ => Err(anyhow::anyhow!("Invalid HodlState")),
        }
    }
}
impl TryInto<String> for HodlState {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            HodlState::OPEN => Ok("OPEN".to_string()),
            HodlState::ACCEPTED => Ok("ACCEPTED".to_string()),
            HodlState::CANCELED => Ok("CANCELED".to_string()),
            HodlState::SETTLED => Ok("SETTLED".to_string()),
        }
    }
}

impl Display for HodlState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvoicePaymentState {
    #[serde(rename = "IN_FLIGHT")]
    InFlight,
    #[serde(rename = "SUCCEEDED")]
    Succeeded,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "INITIATED")]
    Initiaited,
}
impl Display for InvoicePaymentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndPaymentResponse {
    payment_preimage: String,
    status: InvoicePaymentState,
}
impl LndPaymentResponse {
    pub fn preimage(&self) -> String {
        self.payment_preimage.clone()
    }
    pub fn status(&self) -> InvoicePaymentState {
        self.status.clone()
    }
}
impl TryFrom<String> for LndPaymentResponse {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndPaymentResponse {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndPaymentResponse {
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LndPaymentRequest {
    payment_request: String,  // String
    timeout_seconds: i32,     // Int32
    fee_limit_sat: String,    // Int64
    allow_self_payment: bool, // Bool
}
impl LndPaymentRequest {
    pub fn new(
        payment_request: String,
        timeout_seconds: i32,
        fee_limit_sat: String,
        allow_self_payment: bool,
    ) -> Self {
        Self {
            payment_request,
            timeout_seconds,
            fee_limit_sat,
            allow_self_payment,
        }
    }
}
impl ToString for LndPaymentRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl Into<String> for LndPaymentRequest {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
impl TryFrom<String> for LndPaymentRequest {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
