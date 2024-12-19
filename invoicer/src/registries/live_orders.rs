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
    pub fn get_order(&self, order_id: &str) -> Option<OrderInvoiceState> {
        self.orders.get(order_id).cloned()
    }
    pub fn new_order(&mut self, order_id: String, order: OrderInvoiceState) -> anyhow::Result<()> {
        if let Some(old_order) = self.orders.get_mut(&order_id) {
            return Err(anyhow!("Order already exists: {:?}", old_order));
        } else {
            self.orders.insert(order_id, order);
        }
        Ok(())
    }
    pub fn get_mut_order(&mut self, order_id: &str) -> Option<&mut OrderInvoiceState> {
        self.orders.get_mut(order_id)
    }
}
