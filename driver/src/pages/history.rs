use fuente::contexts::LanguageConfigsStore;
use fuente::mass::{OrderDetails, OrderHistoryDesktop, OrderHistoryMobile};
use fuente::models::OrderStatus;
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::contexts::OrderHubStore;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let order_ctx = use_context::<OrderHubStore>().expect("IdbStoreManager not found");
    let orders = order_ctx.order_history();
    let selected_order = use_state(|| None::<String>);
    let date_filter = use_state(|| None::<web_sys::js_sys::Date>);

    let mut completed_orders = orders
        .iter()
        .cloned()
        .filter_map(|order| {
            if let Some(date_filter) = date_filter.as_ref() {
                let order_date_day = web_sys::js_sys::Date::new(&JsValue::from_f64(
                    (order.0.order_timestamp() * 1000) as f64,
                ));
                match order_date_day.get_utc_date() == date_filter.get_utc_date()
                    && order_date_day.get_utc_month() == date_filter.get_utc_month()
                    && order.0.order_status == OrderStatus::Completed
                    && order_date_day.get_utc_full_year() == date_filter.get_utc_full_year()
                {
                    true => Some(order.0),
                    false => None,
                }
            } else {
                match order.0.order_status == OrderStatus::Completed {
                    true => Some(order.0),
                    false => None,
                }
            }
        })
        .collect::<Vec<_>>();
    let mut canceled_orders = orders
        .iter()
        .cloned()
        .filter_map(|order| {
            if let Some(date_filter) = date_filter.as_ref() {
                let order_date_day = web_sys::js_sys::Date::new(&JsValue::from_f64(
                    (order.0.order_timestamp() * 1000) as f64,
                ));
                match order_date_day.get_utc_date() == date_filter.get_utc_date()
                    && order_date_day.get_utc_month() == date_filter.get_utc_month()
                    && order.0.order_status == OrderStatus::Canceled
                    && order_date_day.get_utc_full_year() == date_filter.get_utc_full_year()
                {
                    true => Some(order.0),
                    false => None,
                }
            } else {
                match order.0.order_status == OrderStatus::Canceled {
                    true => Some(order.0),
                    false => None,
                }
            }
        })
        .collect::<Vec<_>>();
    completed_orders.sort_by(|a, b| b.order_timestamp().cmp(&a.order_timestamp()));
    canceled_orders.sort_by(|a, b| b.order_timestamp().cmp(&a.order_timestamp()));

    if let Some(order_id) = (*selected_order).clone() {
        if let Some(order) = orders.iter().find(|o| o.0.order_id() == order_id) {
            return html! {
                <OrderDetails
                    order={order.0.clone()}
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
                      {&translations["orders_heading"]}
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
