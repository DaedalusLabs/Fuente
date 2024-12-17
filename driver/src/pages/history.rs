use fuente::{
    mass::HistoryIcon,
    models::{OrderStatus, OrderInvoiceState},
};
use fuente::models::OrderStateIdb;
use nostr_minions::{browser_api::IdbStoreManager, key_manager::NostrIdStore};
use yew::prelude::*;
use fuente::models::DRIVER_HUB_PRIV_KEY;

#[derive(Clone, PartialEq)]
enum HistoryFilter {
    Completed,
    Canceled,
}

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let orders_state = use_state(|| Vec::<OrderInvoiceState>::new());
    let filter_state = use_state(|| HistoryFilter::Completed);
    let selected_order = use_state(|| None::<String>);
    
    let orders = orders_state.clone();
    use_effect_with((), {
        let orders = orders.clone();
        move |_| {
            if let Some(keys) = key_ctx.get_nostr_key() {
                yew::platform::spawn_local(async move {
                    // Getting all orders from IDB directly
                    match OrderStateIdb::retrieve_all_from_store().await {
                        Ok(all_orders) => {
                            let mut history_orders = Vec::new();
                            
                            for order_entry in all_orders {
                                let state_note = order_entry.signed_note();
                                let hub_keys = nostro2::keypair::NostrKeypair::new(DRIVER_HUB_PRIV_KEY)
                                .expect("Failed to create hub keys");
                                    
                                if let Ok(decrypted) = hub_keys.decrypt_nip_04_content(&state_note) {
                                    if let Ok(order_state) = OrderInvoiceState::try_from(decrypted) {
                                        if let Some(courier) = order_state.get_courier() {
                                            if courier.pubkey == keys.public_key() {
                                                let status = order_state.get_order_status();
                                                if status == OrderStatus::Completed || status == OrderStatus::Canceled {
                                                    gloo::console::log!("Added to history with status:", status.display());
                                                    history_orders.push(order_state);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    gloo::console::warn!("Failed to decrypt order:", order_entry.id());
                                }
                            }
                            orders.set(history_orders);
                        }
                        Err(e) => {
                            gloo::console::error!("Failed to retrieve orders from IDB:", e);
                        }
                    }
                });
            }
            || {}
        }
    });

    let filtered_orders = (*orders).iter()
        .filter(|order| {
            let status = order.get_order_status();
            let matches = match *filter_state {
                HistoryFilter::Completed => status == OrderStatus::Completed,
                HistoryFilter::Canceled => status == OrderStatus::Canceled,
            };
            matches
        })
        .collect::<Vec<_>>();

    if let Some(order_id) = (*selected_order).clone() {
        if let Some(order) = orders.iter().find(|o| o.id() == order_id) {
            return html! {
                <OrderDetails 
                    order={order.clone()} 
                    on_back={Callback::from({
                        let selected = selected_order.clone();
                        move |_| selected.set(None)
                    })}
                />
            };
        }
    }

    html! {
        <div class="flex flex-col flex-1">
            <div class="flex flex-row justify-between items-center p-7">
                <h2 class="text-4xl font-mplus text-fuente-dark">{"Order History"}</h2>
                <div class="flex gap-2">
                    <button 
                        onclick={Callback::from({
                            let filter = filter_state.clone();
                            move |_| filter.set(HistoryFilter::Completed)
                        })}
                        class={classes!(
                            if *filter_state == HistoryFilter::Completed { 
                                "bg-fuente text-white" 
                            } else { 
                                "bg-gray-200" 
                            },
                            "px-4",
                            "py-2",
                            "rounded-lg"
                        )}
                    >
                        {"Completed"}
                    </button>
                    <button 
                        onclick={Callback::from({
                            let filter = filter_state.clone();
                            move |_| filter.set(HistoryFilter::Canceled)
                        })}
                        class={classes!(
                            if *filter_state == HistoryFilter::Canceled { 
                                "bg-fuente text-white" 
                            } else { 
                                "bg-gray-200" 
                            },
                            "px-4",
                            "py-2",
                            "rounded-lg"
                        )}
                    >
                        {"Canceled"}  
                    </button>
                </div>
            </div>

            {if filtered_orders.is_empty() {
                html! {
                    <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                        <HistoryIcon class="w-32 h-32 stroke-neutral-200" />
                        <h4 class="text-xl font-semibold mt-4">{"No deliveries yet"}</h4>
                        <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48 text-center text-wrap">
                            {"Your completed deliveries will appear here"}
                        </p>
                    </div>
                }
            } else {
                html! {
                    <div class="flex flex-col w-full h-full gap-4 p-4 overflow-y-auto">
                        {filtered_orders.iter().map(|order| {
                            let order_req = order.get_order_request();
                            let profile = order_req.profile;
                            let order_id = order.id();
                            let selected = selected_order.clone();
                            
                            html! {
                                <div 
                                    onclick={Callback::from(move |_| selected.set(Some(order_id.clone())))}
                                    class="flex flex-col p-4 border rounded-lg cursor-pointer hover:bg-gray-50 shadow-lg"
                                >
                                    <div class="flex justify-between items-center">
                                        <div>
                                            <h4 class="font-semibold">{profile.nickname}</h4>
                                            <p class="text-sm text-gray-500">
                                                {format!("Order #{}", &order.id()[..8])}
                                            </p>
                                            <p class="text-sm text-gray-500">
                                                {order_req.address.lookup().display_name()}
                                            </p>
                                        </div>
                                        <div class="text-right">
                                            <p class="text-sm font-medium">
                                                {format!("{:.2} SRD", order_req.products.total())}
                                            </p>
                                            <p class={classes!(
                                                "text-sm",
                                                if order.get_order_status() == OrderStatus::Completed {
                                                    "text-green-600"
                                                } else {
                                                    "text-red-600"
                                                }
                                            )}>
                                                {order.get_order_status().display()}
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }
            }}
        </div>
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
                    {format!("Delivery Details #{}", &props.order.id()[..8])}
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
                            if props.order.get_order_status() == OrderStatus::Completed {
                                "text-green-600"
                            } else {
                                "text-red-600"
                            }
                        )}>
                            {props.order.get_order_status().display()}
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
