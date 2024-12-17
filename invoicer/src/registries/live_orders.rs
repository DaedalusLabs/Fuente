use std::collections::HashMap;

use anyhow::anyhow;
use fuente::models::OrderInvoiceState;

#[derive(Debug, Clone)]
pub struct LiveOrders {
    orders: HashMap<String, OrderInvoiceState>,
}
impl Default for LiveOrders {
    fn default() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }
}
impl LiveOrders {
    pub fn get_order(&self, commerce_id: &str) -> Option<OrderInvoiceState> {
        self.orders.get(commerce_id).cloned()
    }
    pub fn new_order(
        &mut self,
        commerce_id: String,
        order: OrderInvoiceState,
    ) -> anyhow::Result<()> {
        if let Some(old_order) = self.orders.get_mut(&commerce_id) {
            return Err(anyhow!("Order already exists: {:?}", old_order));
        } else {
            self.orders.insert(commerce_id, order);
        }
        Ok(())
    }
}
