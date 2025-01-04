use fuente::{
    mass::templates::OrderHistoryTemplate,
    models::{OrderInvoiceState, OrderStateIdb},
};
use yew::prelude::*;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let orders_state = use_state(|| Vec::<OrderInvoiceState>::new());

    let orders = orders_state.clone();
    use_effect_with((), move |_| {
        let orders = orders.clone();
        yew::platform::spawn_local(async move {
            match OrderStateIdb::find_history().await {
                Ok(found_orders) => {
                    orders.set(found_orders);
                }
                Err(e) => {
                    gloo::console::error!("Failed to load orders:", e);
                }
            }
        });
        || {}
    });

    html! {
        <OrderHistoryTemplate orders={(*orders_state).clone()} />
    }
}
