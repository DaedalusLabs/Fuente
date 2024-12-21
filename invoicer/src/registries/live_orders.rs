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
    pub fn update_order_record(
        &mut self,
        order_id: String,
        order: OrderInvoiceState,
    ) -> anyhow::Result<()> {
        self.orders.insert(order_id, order);
        Ok(())
    }
    pub fn remove_order(&mut self, order_id: &str) -> anyhow::Result<()> {
        self.orders
            .remove(order_id)
            .ok_or_else(|| anyhow!("Order not found"))?;
        Ok(())
    }
}
