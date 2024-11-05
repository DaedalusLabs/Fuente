use serde::{Deserialize, Serialize};

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
