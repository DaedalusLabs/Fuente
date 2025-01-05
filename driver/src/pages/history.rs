use fuente::mass::templates::OrderHistoryTemplate;
use fuente::models::{OrderInvoiceState, OrderStatus};
use yew::prelude::*;

use crate::contexts::OrderHubStore;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let order_ctx = use_context::<OrderHubStore>().expect("IdbStoreManager not found");

    let mut filtered_orders = order_ctx
        .order_history()
        .iter()
        .map(|order| order.0.clone())
        .collect::<Vec<_>>();

    filtered_orders.sort_by(|a, b| b.order.created_at.cmp(&a.order.created_at));

    html! {
        <OrderHistoryTemplate orders={filtered_orders.clone()} />
    }
}

#[derive(Properties, Clone, PartialEq)]
struct OrderDetailsProps {
    order: OrderInvoiceState,
    on_back: Callback<MouseEvent>,
}

#[function_component(OrderDetails)]
fn order_details(props: &OrderDetailsProps) -> Html {
    let order_req = props.order.get_order_request();
    let products = order_req.products.counted_products();

    html! {
        <div class="flex flex-col w-full h-full">
            <div class="flex items-center gap-4 mb-6 p-6">
                <button
                    onclick={props.on_back.clone()}
                    class="p-2 rounded-lg hover:bg-gray-100"
                >
                    {"← Back"}
                </button>
                <h2 class="text-2xl font-semibold text-fuente-dark">
                    {format!("Delivery Details #{}", &props.order.order_id()[..8])}
                </h2>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-8 p-6">
                <div class="space-y-6">
                    <div class="space-y-2">
                        <h3 class="font-medium text-gray-500">{"Customer Information"}</h3>
                        <div class="space-y-1">
                            <p>{format!("Name: {}", order_req.profile.nickname)}</p>
                            <p>{format!("Phone: {}", order_req.profile.telephone)}</p>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <h3 class="font-medium text-gray-500">{"Delivery Address"}</h3>
                        <p class="text-sm">{order_req.address.lookup().display_name()}</p>
                    </div>

                    <div class="space-y-2">
                        <h3 class="font-medium text-gray-500">{"Delivery Status"}</h3>
                        <p class={classes!(
                            "font-medium",
                            if props.order.order_status == OrderStatus::Completed {
                                "text-green-600"
                            } else {
                                "text-red-600"
                            }
                        )}>
                            {props.order.order_status.display()}
                        </p>
                    </div>
                </div>

                <div class="space-y-4">
                    <h3 class="font-medium text-gray-500">{"Order Items"}</h3>
                    <div class="space-y-2">
                        {products.iter().map(|(product, count)| {
                            let subtotal = product.price().parse::<f64>().unwrap() * *count as f64;
                            html! {
                                <div class="flex justify-between py-2 border-b">
                                    <div>
                                        <p class="font-medium">{product.name()}</p>
                                        <p class="text-sm text-gray-500">
                                            {format!("{} x {} SRD", count, product.price())}
                                        </p>
                                    </div>
                                    <p class="font-medium">{format!("{:.2} SRD", subtotal)}</p>
                                </div>
                            }
                        }).collect::<Html>()}

                        <div class="flex justify-between pt-4 font-medium">
                            <p>{"Total"}</p>
                            <p>{format!("{:.2} SRD", order_req.products.total())}</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
