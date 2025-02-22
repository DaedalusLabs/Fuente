use lucide_yew::{Check, Clock, Hammer, MapPinCheck, Truck, X};
use nostr_minions::key_manager::NostrIdStore;
use yew::prelude::*;

use crate::{
    contexts::LanguageConfigsStore,
    mass::{CustomerDetails, ProductListItem},
    models::{DriverProfile, OrderInvoiceState, OrderStatus},
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
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let OrderDetailModalProps { order, on_submit } = props;
    let is_customer = {
        let keypair = key_ctx.get_nostr_key().expect("Nostr key not found");
        let pubkey = keypair.public_key();
        order.order.pubkey == pubkey
    };
    let request = order.get_order_request();
    let products = request.products.counted_products();
    let order_total = request.products.total();
    let customer_profile = &request.profile;
    let order_status = order.order_status.clone();
    let driver_profile = {
        let driver = order
            .courier
            .as_ref()
            .map(|courier| DriverProfile::try_from(courier));
        driver
    };
    let icon_class = classes!("w-6", "h-6", order_status.text_color());
    html! {
        <>
            <div class="flex items-center justify-between border-b border-b-gray-400 pb-3 gap-2 sm:gap-4 md:gap-6">
                <div>
                    <p class="text-fuente-dark font-bold text-2xl">{format!("#{}", &order.order_id()[..12])}</p>
                    <p class="text-gray-500 font-light text-lg">{&translations["store_order_modal_title"]}</p>
                </div>
                <button
                    class={classes!(
                        "border-2",
                        "bg-white",
                        "rounded-2xl",
                        "py-3",
                        "px-4",
                        "text-center",
                        "font-semibold",
                        order_status.text_color(),
                        order_status.border_color()
                    )}>
                    {
                        match order_status {
                            OrderStatus::Pending => html! {<Clock class={icon_class} />},
                            OrderStatus::Preparing => html! {<Hammer class={icon_class} />},
                            OrderStatus::ReadyForDelivery => html! {<MapPinCheck class={icon_class} />},
                            OrderStatus::InDelivery => html! {<Truck class={icon_class} />},
                            OrderStatus::Completed => html! {<Check class={icon_class} />},
                            OrderStatus::Canceled => html! {<X class={icon_class} />},
                        }
                    }
                </button>
            </div>

            <h3 class="text-gray-500 mt-5 font-light">{&translations["store_order_modal_products"]}</h3>
            {products.iter().map(|(product, count)| {
                html! {
                    <ProductListItem product={product.clone()} count={*count} />
                }
            }).collect::<Html>()}

            <div class="my-5 bg-gray-200 flex justify-end p-3">
                <div class="space-y-2">
                    <p class="text-fuente font-bold text-lg text-right">{format!("SRD {}", order_total)}</p>
                </div>
            </div>

            <CustomerDetails customer={customer_profile.clone()} />
            {if let Some(Ok(driver)) = driver_profile {
                html! {
                    <div class="mt-5">
                        <h3 class="text-gray-500 font-light">{&translations["packages_track_table_heading_driver"]}</h3>
                        <div class="flex items-center gap-3">
                            <div>
                                <p class="text-fuente font-bold text-lg">{driver.nickname()}</p>
                                <p class="text-gray-500 font-light text-md">{driver.telephone()}</p>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {<></>}
            }}
            {if !is_customer {
                html! {
                    <OrderModalForm current_status={order.order_status.clone()} on_order_click={on_submit.clone()} />
                }
            } else {
                html! {<></>}
            }}
        </>
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

    let cancel_button_text = match current_status {
        OrderStatus::Pending => translations["store_order_action_reject"].clone(),
        OrderStatus::Preparing => translations["store_order_action_cancel"].clone(),
        OrderStatus::ReadyForDelivery => translations["store_order_action_cancel"].clone(),
        _ => String::new(),
    };
    let cancel_form = html! {
        <form onsubmit={on_order_click.clone()}>
            <input type="hidden" name="order_status" value={OrderStatus::Canceled.to_string()} />
            // <div class="mb-4">
            //     <label for="cancel_reason" class="block text-gray-500 text-sm font-bold mb-2">
            //     {&translations["store_order_action_reject_reason"]}
            //     </label>
            //     <textarea
            //         id="cancel_reason"
            //         name="cancel_reason"
            //         class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            //         required={true}
            //         rows="3"
            //         placeholder={translations["store_order_action_reject_reason"].clone()}
            //     />
            // </div>
            <button
                type="submit"
                class="border-2 border-red-500 text-red-500 bg-white text-center text-lg font-bold rounded-full w-full py-3 hover:bg-red-50"
            >
                {cancel_button_text}
            </button>
        </form>
    };

    let order_button_text = match current_status {
        OrderStatus::Pending => translations["store_order_action_accept"].clone(),
        OrderStatus::Preparing => translations["store_order_action_deliver"].clone(),
        _ => String::new(),
    };
    match current_status {
        OrderStatus::Pending => {
            html! {
                <div class="mt-5 space-y-4">
                    <form onsubmit={on_order_click.clone()}>
                        <input type="hidden" name="order_status" value={OrderStatus::Preparing.to_string()} />
                        <input type="submit" value={order_button_text.clone()}
                        class="bg-fuente-orange text-white text-center text-lg font-bold rounded-full w-full py-3 mt-5 cursor-pointer" />
                    </form>
                    {cancel_form}
                </div>
            }
        }
        OrderStatus::Preparing => {
            html! {
                <div class="mt-5 space-y-4">
                    <form onsubmit={on_order_click.clone()}>
                        <input type="hidden" name="order_status" value={OrderStatus::ReadyForDelivery.to_string()} />
                        <button
                            type="submit"
                            class="bg-sky-500 text-white text-center text-lg font-bold rounded-full w-full py-3"
                        >
                            {order_button_text}
                        </button>
                    </form>
                    {cancel_form}
                </div>
            }
        }
        OrderStatus::ReadyForDelivery => {
            html! {
                <div class="mt-5">
                    {cancel_form}
                </div>
            }
        }
        OrderStatus::InDelivery => html! {},
        _ => html! {},
    }
}
