use lucide_yew::{Clock, Hammer, MapPinCheck, Truck};
use yew::prelude::*;

use crate::{
    contexts::LanguageConfigsStore,
    mass::{CustomerDetails, ProductListItem},
    models::{OrderInvoiceState, OrderStatus},
};
#[derive(Clone, PartialEq, Properties)]
pub struct OrderDetailModalProps {
    pub order: OrderInvoiceState,
    pub on_submit: Callback<SubmitEvent>,
}

#[function_component(OrderDetailModal)]
pub fn order_detail_modal(props: &OrderDetailModalProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let OrderDetailModalProps { order, on_submit } = props;
    let request = order.get_order_request();
    let products = request.products.counted_products();
    let order_total = request.products.total();
    let customer_profile = &request.profile;
    let order_status = order.order_status.clone();
    html! {
        <main class="bg-white rounded-2xl py-auto px-auto">
            <div class="flex items-center justify-between border-b border-b-gray-400 pb-3 gap-2">
                <div>
                    <p class="text-fuente-dark font-bold text-2xl">{format!("#{}", &order.order_id()[..12])}</p>
                    <p class="text-gray-500 font-light text-lg">{&translations["store_order_modal_title"]}</p>
                </div>
                <button
                    class={classes!("border-2", "bg-white", "rounded-2xl", "py-3", "px-4", "text-center", "font-semibold",
                        order_status.text_color(), order_status.border_color())}>
                        {match order_status {
                            OrderStatus::Pending => html! {<Clock class="w-6 h-6" />},
                            OrderStatus::Preparing => html! {<Hammer class="w-6 h-6" />},
                            OrderStatus::ReadyForDelivery => html! {<MapPinCheck class="w-6 h-6" />},
                            OrderStatus::InDelivery => html! {<Truck class="w-6 h-6" />},
                            _ => {html! {}},
                        }}
                </button>
            </div>

            <h3 class="text-gray-500 mt-5 font-light">{&translations["store_order_modal_products"]}</h3>
            {products.iter().map(|(product, count)| {
                html! {
                    <ProductListItem product={product.clone()} count={*count} />
                }
            }).collect::<Html>()}

            <div class="mt-5 bg-gray-200 flex justify-end p-3">
                <div class="space-y-2">
                    <p class="text-fuente font-bold text-lg text-right">{format!("SRD {}", order_total)}</p>
                </div>
            </div>

            <CustomerDetails customer={customer_profile.clone()} />
            <OrderModalForm current_status={order.order_status.clone()} on_order_click={on_submit.clone()} />
        </main>
    }
}
#[derive(Clone, PartialEq, Properties)]
pub struct OrderModalFormProps {
    pub current_status: OrderStatus,
    pub on_order_click: Callback<SubmitEvent>,
}
#[function_component(OrderModalForm)]
pub fn order_modal_form(props: &OrderModalFormProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let OrderModalFormProps {
        current_status,
        on_order_click,
    } = props;
    let options = match current_status {
        OrderStatus::Pending => Some(html! {
            <>
                <option class={OrderStatus::Preparing.theme_color()} value={OrderStatus::Preparing.to_string()}>{OrderStatus::Preparing.display()}</option>
                <option class={OrderStatus::Canceled.theme_color()} value={OrderStatus::Canceled.to_string()}>{OrderStatus::Canceled.display()}</option>
            </>
        }),
        OrderStatus::Preparing => Some(html! {
            <>
                <option value={OrderStatus::ReadyForDelivery.to_string()}>{OrderStatus::ReadyForDelivery.display()}</option>
                <option value={OrderStatus::Canceled.to_string()}>{OrderStatus::Canceled.display()}</option>
            </>
        }),
        OrderStatus::ReadyForDelivery => Some(html! {
            <option value={OrderStatus::Canceled.to_string()}>{OrderStatus::Canceled.display()}</option>
        }),
        _ => None,
    };
    match options {
        Some(options) => {
            html! {
                <form onsubmit={on_order_click.clone()} class="mt-5">
                    <div class="flex justify-between items-center">
                        <label for="order_status" class="text-gray-500 font-light text-lg w-full">{&translations["store_order_modal_option_response"]}</label>
                        <select id="order_status" name="order_status" class="py-3 px-5 rounded-xl border border-gray-500 w-full text-gray-500">
                            {options}
                        </select>
                    </div>
                    <input type="submit" value={translations["store_order_modal_button_submit"].clone()}
                        class="bg-fuente-orange text-white text-center text-lg font-bold rounded-full w-full py-3 mt-5" />
                </form>
            }
        }
        None => html! { <></> },
    }
}
