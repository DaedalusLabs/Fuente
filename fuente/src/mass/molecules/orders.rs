use yew::prelude::*;

use crate::{
    contexts::LanguageConfigsStore,
    models::{ConsumerProfile, OrderInvoiceState, OrderStatus, ProductItem},
};

#[derive(Clone, PartialEq, Properties)]
pub struct OrderStateCardProps {
    pub order: OrderInvoiceState,
    pub on_click: Callback<MouseEvent>,
}

#[function_component(OrderStateCard)]
pub fn order_state_card(props: &OrderStateCardProps) -> Html {
    let order = &props.order;
    let order_req = order.get_order_request();
    let profile = order_req.profile;
    let order_id = order.order_id();
    let timestamp = web_sys::js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
        order.order_timestamp() as f64 * 1000.0,
    ));
    let locale_options = web_sys::js_sys::Object::new();
    let locale_options = web_sys::js_sys::Intl::DateTimeFormat::new(
        &web_sys::js_sys::Array::of1(&"nl-SR".into()),
        &locale_options,
    );
    let locale_date = timestamp.to_locale_date_string("nl-SR", &locale_options);
    let locale_time = timestamp.to_locale_time_string("nl-SR");
    html! {
        <div onclick={props.on_click.clone()} id={order_id} class="bg-white shadow py-2 px-5 rounded-2xl space-y-1">
            <p class="text-fuente font-bold text-md">{profile.nickname}</p>
            <p class="font-bold text-sm">{format!("#{}", &order.order_id()[..8])}</p>
            <p class="text-gray-500 text-xs">{format!("{} | {}", locale_date, locale_time)}</p>
        </div>
    }
}
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
    html! {
        <main class="bg-white rounded-2xl py-5 px-10 max-w-xl ml-auto max-h-screen">
            <div class="flex items-center justify-between border-b border-b-gray-400 pb-3">
                <div>
                    <p class="text-fuente-dark font-bold text-2xl">{format!("#{}", &order.order_id()[..12])}</p>
                    <p class="text-gray-500 font-light text-lg">{&translations["store_order_modal_title"]}</p>
                </div>
                <button
                    class="border-2 border-gray-400 text-gray-400 bg-white rounded-2xl py-3 px-4 text-center font-semibold">{order.order_status.to_string()}</button>
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

#[derive(Clone, PartialEq, Properties)]
pub struct CustomerDetailsProps {
    pub customer: ConsumerProfile,
}
#[function_component(CustomerDetails)]
pub fn customer_details(props: &CustomerDetailsProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let CustomerDetailsProps { customer } = props;
    html! {
        <section class="mt-5 space-y-3 border-y border-y-gray-400 py-3">
            <h3 class="text-gray-500 font-light text-lg">{&translations["store_order_modal_customer"]}</h3>
            <p class="text-gray-500 font-bold text-lg">{&customer.nickname}</p>
            <div class="w-96 space-y-2">
                <div class="flex justify-between">
                    <p class="text-gray-500 font-bold text-lg">{&translations["checkout_client_information_heading_email"]}</p>
                    <p class="text-gray-500">{&customer.email}</p>
                </div>

                <div class="flex justify-between">
                    <p class="text-gray-500 font-bold text-lg">{&translations["checkout_client_information_heading_phone"]}</p>
                    <p class="text-gray-500">{&customer.telephone}</p>
                </div>
            </div>
        </section>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ProductListItemProps {
    pub product: ProductItem,
    pub count: u32,
}
#[function_component(ProductListItem)]
pub fn product_list_item(props: &ProductListItemProps) -> Html {
    let ProductListItemProps { product, count } = props;
    html! {
        <div class="mt-5 space-y-3 flex items-center justify-between">
            <div class="flex items-center gap-5">
                <img src={product.thumbnail_url()} alt={product.name()} class="w-20 block object-contain" />
                <div>
                    <p class="text-gray-500 font-bold text-md">{product.name()}</p>
                    <p class="text-gray-500 font-light line-clamp-3">{product.details()}</p>
                    <p class="text-gray-500 font-bold text-md uppercase">{product.sku()}</p>
                </div>
            </div>

            <div class="flex flex-col items-center gap-2">
                <p class="text-gray-500 font-bold text-xl">{product.price()}</p>
                <p class="text-gray-500 font-bold text-md">{format!("x{}", count)}</p>
            </div>
        </div>
    }
}
