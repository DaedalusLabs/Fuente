use crate::contexts::OrderDataStore;
use fuente::{
    contexts::LanguageConfigsStore,
    mass::OrderStateCard,
    models::{OrderInvoiceState, OrderStatus},
};
use lucide_yew::{ArrowLeft, History};
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let order_ctx = use_context::<OrderDataStore>().expect("No order context found");
    let orders = order_ctx.order_history();
    let selected_order = use_state(|| None::<String>);
    let date_filter = use_state(|| None::<web_sys::js_sys::Date>);

    let mut completed_orders = orders
        .iter()
        .cloned()
        .filter(|order| {
            let date_filtered = {
                if let Some(date_filter) = &*date_filter {
                    let order_date_day = web_sys::js_sys::Date::new(&JsValue::from_f64(
                        (order.order_timestamp() * 1000) as f64,
                    ));
                    order_date_day.get_utc_date() == date_filter.get_utc_date()
                        && order_date_day.get_utc_month() == date_filter.get_utc_month()
                        && order_date_day.get_utc_full_year() == date_filter.get_utc_full_year()
                } else {
                    true
                }
            };
            date_filtered && order.order_status == OrderStatus::Completed
        })
        .collect::<Vec<_>>();
    let mut canceled_orders = orders
        .iter()
        .cloned()
        .filter(|order| {
            let date_filtered = {
                if let Some(date_filter) = &*date_filter {
                    let order_date_day = web_sys::js_sys::Date::new(&JsValue::from_f64(
                        (order.order_timestamp() * 1000) as f64,
                    ));
                    order_date_day.get_utc_date() == date_filter.get_utc_date()
                        && order_date_day.get_utc_month() == date_filter.get_utc_month()
                        && order_date_day.get_utc_full_year() == date_filter.get_utc_full_year()
                } else {
                    true
                }
            };
            date_filtered && order.order_status == OrderStatus::Canceled
        })
        .collect::<Vec<_>>();
    completed_orders.sort_by(|a, b| b.order_timestamp().cmp(&a.order_timestamp()));
    canceled_orders.sort_by(|a, b| b.order_timestamp().cmp(&a.order_timestamp()));

    if let Some(order_id) = (*selected_order).clone() {
        if let Some(order) = orders.iter().find(|o| o.order_id() == order_id) {
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

    let onchange = {
        let date_filter = date_filter.clone();
        Callback::from(move |e: Event| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            let value = web_sys::js_sys::Date::new(&JsValue::from_str(&value));
            date_filter.set(Some(value));
        })
    };

    html! {
          <main class="flex-1 flex flex-col h-screen overflow-hidden">
              <div class="flex flex-row justify-between items-center p-4 lg:p-10">
                  <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-0 lg:text-6xl tracking-tighter font-bold">
                      {&translations["store_orders_history_title"]}
                  </h1>
                  <div class="flex items-center gap-5">
                      <label for="date" class="hidden lg:block text-fuente font-light text-md w-full text-right">{&translations["store_orders_history_date"]}</label>
                      <div class="relative w-fit">
                          <input {onchange} type="date" placeholder=""
                            class="border-2 border-fuente w-full rounded-xl py-2 px-5 text-fuente placeholder:text-fuente" id="date" />
                      </div>
                  </div>
              </div>
              <OrderHistoryMobile
                  completed_orders={completed_orders.clone()}
                  canceled_orders={canceled_orders.clone()}
                  on_order_click={
                    Callback::from({
                        let selected = selected_order.clone();
                        move |e: MouseEvent| {
                            let target = e.target_unchecked_into::<HtmlInputElement>();
                            let order_id = target.id();
                            selected.set(Some(order_id));
                        }
                    })
                }
                  />
              <OrderHistoryDesktop
                  completed_orders={completed_orders}
                  canceled_orders={canceled_orders}
                  on_order_click={
                    Callback::from({
                        let selected = selected_order.clone();
                        move |e: MouseEvent| {
                            let target = e.target_unchecked_into::<HtmlInputElement>();
                            let order_id = target.id();
                            selected.set(Some(order_id));
                        }
                    })
                }
                  />
        </main>

    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct OrderHistoryProps {
    completed_orders: Vec<OrderInvoiceState>,
    canceled_orders: Vec<OrderInvoiceState>,
    on_order_click: Callback<MouseEvent>,
}

#[function_component(OrderHistoryMobile)]
pub fn order_history_desktop(props: &OrderHistoryProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let filter = use_state(|| OrderStatus::Completed);
    let OrderHistoryProps {
        completed_orders,
        canceled_orders,
        on_order_click,
    } = props;
    let bg_color = match *filter {
        OrderStatus::Completed => "bg-green-100",
        OrderStatus::Canceled => "bg-red-100",
        _ => "bg-gray-100",
    };
    html! {
        <div class="flex lg:hidden flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                <div class="flex flex-col gap-2 w-full h-full overflow-hidden">
                    <div class="grid grid-flow-col justify-stretch gap-2 w-full">
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(OrderStatus::Completed)
                            })}
                            class={classes!("border-green-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                            <p class={classes!("text-lg", "font-semibold", "text-center", "text-green-500")}>
                                {&translations["store_orders_history_completed"]}
                            </p>
                        </div>
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(OrderStatus::Canceled)
                            })}
                            class={classes!("border-red-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                            <p class={classes!("text-lg", "font-semibold", "text-center", "text-red-500")}>
                                {&translations["store_orders_history_canceled"]}
                            </p>
                        </div>
                    </div>
                    <div
                        class={classes!("flex-1", "rounded-2xl", "mt-2", "px-2", "py-2", "overflow-y-auto", "no-scrollbar", bg_color)}>
                        <div class="grid grid-cols-1 gap-4">
                        {
                            match *filter {
                                OrderStatus::Completed => completed_orders.iter().map(|order| {
                                    html! { <OrderStateCard order={(*order).clone()} on_click={on_order_click.clone()} />}
                                }).collect::<Html>(),
                                OrderStatus::Canceled => canceled_orders.iter().map(|order| {
                                    html! { <OrderStateCard order={(*order).clone()} on_click={Callback::noop()} />}
                                }).collect::<Html>(),
                                _ => html! {},
                            }
                        }
                        </div>
                    </div>

                </div>
            </div>
        </div>
    }
}

#[function_component(OrderHistoryDesktop)]
pub fn order_history_desktop(props: &OrderHistoryProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let OrderHistoryProps {
        completed_orders,
        canceled_orders,
        on_order_click: _,
    } = props;
    html! {
        <div class="hidden lg:flex flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                <div class="flex flex-col gap-2 w-1/2 h-full overflow-hidden">
                    <div class="border-2 border-green-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-green-500 text-lg font-semibold text-center">{&translations["store_orders_history_completed"]}</p>
                    </div>

                     <div class={"flex-1 rounded-2xl mt-2 px-2 py-2 overflow-y-auto no-scrollbar bg-green-100"}>
                        <div class="grid grid-cols-1 gap-4">
                        {completed_orders.iter().map(|order| {
                           html! {  <OrderStateCard order={(*order).clone()} on_click={Callback::noop()} />}
                        }).collect::<Html>()}
                        </div>
                    </div>

                </div>

                <div class="flex flex-col gap-2 w-1/2 h-full overflow-hidden">
                    <div class="border-2 border-red-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-red-500 text-lg font-semibold text-center">{&translations["store_orders_history_canceled"]}</p>
                    </div>

                     <div class={"flex-1 rounded-2xl mt-2 px-2 py-2 overflow-y-auto no-scrollbar bg-red-100"}>
                        <div class="grid grid-cols-1 gap-4">
                        {canceled_orders.iter().map(|order| {
                            html! { <OrderStateCard order={(*order).clone()} on_click={Callback::noop()} />}
                        }).collect::<Html>()}
                        </div>
                    </div>
                </div>
            </div>
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
        <div class="flex flex-col w-full h-full p-4 max-w-4xl mx-auto">
          <div class="flex items-center gap-4 mb-6">
            <button
                onclick={props.on_back.clone()}
              class="p-2 rounded-lg hover:bg-gray-100 transition-colors duration-200"
            >
              <ArrowLeft class="w-5 h-5" />
            </button>
            <h2 class="text-2xl font-semibold text-fuente-dark">
                {format!("Order Details #{}", props.order.order_id()[..8].to_string())}
            </h2>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div class="space-y-6">
             <div class="space-y-2">
                <h3 class="font-medium text-fuente">{"Customer Information"}</h3>
                <p>{format!("Name: {}", order_req.profile.nickname)}</p>
                <p>{format!("Phone: {}", order_req.profile.telephone)}</p>
                <p>{format!("Email: {}", order_req.profile.email)}</p>
              </div>
            </div>
            <div class="space-y-2">
                <h3 class="font-medium text-fuente">{"Delivery Address"}</h3>
                <p class="text-sm">{order_req.address.lookup().display_name()}</p>
            </div>
            <div class="space-y-2">
                <h3 class="font-medium text-fuente">{"Order Status"}</h3>
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

                <div class="space-y-4">
                    <h3 class="font-medium text-fuente">{"Order Items"}</h3>
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
#[function_component(BlankHistory)]
pub fn history_page() -> Html {
    html! {
        <>
            <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                <History class="w-32 h-32 stroke-neutral-200" />
                <h4 class="text-xl font-semibold mt-4">{"No history yet"}</h4>
                <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48  text-center text-wrap">
                    {"New sales will appear here!"}
                </p>
            </div>
        </>
    }
}
