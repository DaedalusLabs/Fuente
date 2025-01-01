use super::PageHeader;
use fuente::{
    mass::{templates::OrderHistoryTemplate, HistoryIcon, SimpleFormButton},
    models::{OrderInvoiceState, OrderStateIdb, OrderStatus},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum HistoryFilter {
    Completed,
    Canceled,
}

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

#[function_component(HistoryPage2)]
pub fn history_page() -> Html {
    let orders_state = use_state(|| Vec::<OrderInvoiceState>::new());
    let filter_state = use_state(|| HistoryFilter::Completed);
    let selected_order = use_state(|| None::<String>);

    // (Loading orders from IndexedDB)
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

    let filtered_orders = (*orders_state)
        .iter()
        .filter(|order| match *filter_state {
            HistoryFilter::Completed => order.order_status == OrderStatus::Completed,
            HistoryFilter::Canceled => order.order_status == OrderStatus::Canceled,
        })
        .collect::<Vec<_>>();

    if let Some(order_id) = (*selected_order).clone() {
        if let Some(order) = (*orders_state).iter().find(|o| o.order_id() == order_id) {
            return html! {
            };
        }
    }

    if !filtered_orders.is_empty() {
        html! {
            <div class="h-full w-full flex flex-col">
                <div class="flex flex-row justify-between items-center mb-4 p-7">
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

                <div class="flex flex-col gap-4 p-4 overflow-y-auto">
                {for filtered_orders.iter().map(|order| {
                    let order_req = order.get_order_request();
                    let order_id = order.order_id();
                    let selected = selected_order.clone();
                    let profile = order_req.profile;

                    html! {
                        <div
                            onclick={Callback::from(move |_| selected.set(Some(order_id.clone())))}
                            class="flex flex-col p-4 border rounded-lg cursor-pointer hover:bg-gray-50"
                        >
                        <div class="flex justify-between items-center mb-2">
                        <div>
                            <h4 class="font-semibold">{profile.nickname}</h4>
                        </div>
                        <div class="text-right">
                            <p class="text-sm font-medium">
                                {format!("{:.2} SRD", order_req.products.total())}
                            </p>
                        </div>
                    </div>
                    <div class="flex justify-between items-center">
                        <p class="text-sm text-gray-500">
                            {format!("Order #{}", &order.order_id()[..8])}
                        </p>
                        <p class={classes!(
                            "text-sm",
                            if order.order_status == OrderStatus::Completed {
                                "text-green-600"
                            } else {
                                "text-red-600"
                            }
                        )}>
                            {order.order_status.display()}
                                </p>
                            </div>
                        </div>
                    }
                })}
                </div>
            </div>
        }
    } else {
        html! {
            <div class="h-full w-full flex flex-col justify-between items-center">
                <PageHeader title={"History".to_string()} />
                <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                    <HistoryIcon class="w-32 h-32 stroke-neutral-200" />
                    <h4 class="text-xl font-semibold mt-4">{"No history yet"}</h4>
                    <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48 text-center text-wrap">
                        {"Hit the button below to create a new order!"}
                    </p>
                </div>
                <SimpleFormButton>
                    {"Create an Order"}
                </SimpleFormButton>
            </div>
        }
    }
}

