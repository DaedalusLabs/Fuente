use crate::contexts::OrderDataStore;
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{OrderDetails, OrderHistoryDesktop, OrderHistoryMobile},
    models::OrderStatus,
};
use lucide_yew::History;
use web_sys::wasm_bindgen::JsValue;
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
        <main class="flex-1 flex flex-col h-screen overflow-hidden container mx-auto">
            <div class="flex flex-row justify-between items-center">
                <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-10 lg:text-6xl tracking-tighter font-bold font-mplus whitespace-nowrap">
                  {&translations["store_orders_history_title"]}
                </h1>
                <div class="flex items-center gap-5">
                    <label for="date" class="hidden lg:block text-fuente font-light text-md w-full text-right">{&translations["store_orders_history_date"]}</label>
                    <div class="relative w-fit">
                        <input {onchange} type="date" placeholder=""
                            class="border-2 border-fuente max-w-16 sm:max-w-96 rounded-xl py-2 px-5 text-fuente placeholder:text-fuente" id="date" />
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
