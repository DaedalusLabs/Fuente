use crate::{
    mass::{CommerceProfileAddressDetails, CommerceProfileDetails, PopupSection},
    models::{CommerceProfile, ConsumerAddress},
};
use lucide_yew::{Bitcoin, Frown};
use nostr_minions::{
    browser_api::GeolocationCoordinates,
    widgets::leaflet::{IconOptions, LeafletComponent, LeafletMap, LeafletMapOptions, Marker},
};
use nostro2::notes::NostrNote;
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
        <section class="mt-5 space-y-3 border-y border-y-gray-400 py-3 w-full">
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
pub struct CustomerAddressDetailsProps {
    pub customer: ConsumerAddress,
}
#[function_component(CustomerAddressDetails)]
pub fn customer_details(props: &CustomerAddressDetailsProps) -> Html {
    let CustomerAddressDetailsProps { customer } = props;
    html! {
        <section class="space-y-3 border-b border-b-gray-400 py-3 w-full text-wrap">
            <p class="text-gray-500  line-clamp-3">{&customer.lookup().display_name()}</p>
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
#[derive(Clone, PartialEq, Properties)]
pub struct OrderConfirmationProps {
    pub order: OrderInvoiceState,
    pub onclick: Callback<MouseEvent>,
}

#[function_component(OrderPendingTemplate)]
pub fn settings_template() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    html! {
        <>
            <main class="mt-20">
                <div class="w-52 mx-auto flex justify-center items-center">
                    <Bitcoin class="text-fuente-orange" size=128 />
                </div>

                <div class="max-w-md mx-auto mt-5 space-y-3">
                    <h1 class="text-3xl font-bold text-fuente text-center tracking-tighter">
                        {&translations["payment_heading"]}
                    </h1>
                    <p class="font-light text-fuente text-center w-5/6 mx-auto">
                        {&translations["payment_detail"]}
                    </p>
                </div>
            </main>
        </>
    }
}
#[function_component(OrderSuccessTemplate)]
pub fn settings_template(props: &OrderConfirmationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let order = &props.order;
    let onclick = props.onclick.clone();
    html! {
        <>
        <main class="mt-20">
            <div class="w-52 mx-auto flex justify-center items-center">
                <Bitcoin class="text-fuente-orange" size=128 />
            </div>

            <div class="flex flex-col items-center mt-6 space-y-10">
                <div>
                    <h1 class="text-3xl text-fuente text-center font-bold">{&translations["confirmation_heading"]}</h1>
                    <p class="text-3xl text-fuente text-center font-bold">{&translations["confirmation_text"]}</p>
                </div>

                <h3 class="text-fuente-orange font-bold text-2xl">{format!("#{}", &order.order_id()[..8])}</h3>

                <div class="flex flex-col items-center justify-center">
                    <p class="font-light text-center text-fuente">{&translations["confirmation_detail"]}</p>
                    <p class="font-light text-center text-fuente">{&translations["confirmation_detail_message"]}</p>
                    <button {onclick} class="bg-fuente-buttons text-fuente-forms py-4 px-7 rounded-full mt-5 font-bold">
                        {&translations["confirmation_button"]}
                    </button>
                </div>
            </div>
        </main>
        </>
    }
}
#[function_component(OrderFailureTemplate)]
pub fn settings_template(props: &OrderConfirmationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let order = &props.order;
    html! {
        <>
            <main class="mt-20">
                <div class="w-52 mx-auto flex justify-center items-center">
                    <Frown class="text-red-500" size=128 />
                </div>

                <div class="flex flex-col items-center mt-6 space-y-10">
                    <div>
                        <h1 class="text-3xl text-fuente text-center font-bold">{&translations["error_screen_heading"]}</h1>
                        <p class="text-3xl text-fuente text-center font-bold">{&translations["error_screen_text"]}</p>
                    </div>

                    <h3 class="text-fuente font-bold text-2xl">{format!("#{}", &order.order_id()[..8])}</h3>

                    <div class="flex flex-col items-center justify-center">
                        <p class="font-light text-center text-fuente">{&translations["error_screen_detail"]}</p>
                        <button class="bg-fuente-buttons text-fuente-forms py-4 px-7 rounded-full mt-5 font-bold">
                            {&translations["error_screen_detail_message"]}
                        </button>
                    </div>
                </div>
            </main>
        </>
    }
}
#[function_component(CheckoutBannerTemplate)]
pub fn settings_template(props: &OrderConfirmationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let order_state = &props.order.order_status;
    let number_class = classes!(
        "text-white",
        "w-10",
        "h-10",
        "lg:w-16",
        "lg:h-16",
        "rounded-full",
        "font-bold",
        "text-lg",
        "text-center",
        "flex",
        "items-center",
        "justify-center"
    );
    let text_class = classes!("font-bold", "text-md", "md:text-lg", "lg:text-xl");
    let (confirmation_color, confirmation_text) = if order_state == &OrderStatus::Pending {
        ("bg-orange-500", "text-orange-500")
    } else {
        ("bg-fuente", "text-fuente")
    };
    let (payment_color, payment_text) = if order_state != &OrderStatus::Pending {
        ("bg-orange-500", "text-orange-500")
    } else {
        ("bg-fuente", "text-fuente")
    };
    html! {
        <>
        <div class="bg-gray-100 py-5 flex justify-center items-center gap-5 lg:gap-16">
            <div class="flex items-center gap-5">
                <p class={classes!(number_class.clone(), confirmation_color)}>{"1"}</p>
                <p class={classes!(text_class.clone(), confirmation_text)}>{&translations["payment_step_1"]}</p>
            </div>

            <div class="flex items-center justify-end gap-4">
                <p class={classes!(number_class, payment_color)}>{"2"}</p>
                <p class={classes!(text_class, payment_text)}>{&translations["payment_step_2"]}</p>
            </div>
        </div>
        </>
    }
}
#[derive(Clone, PartialEq, Properties)]
pub struct OrderListProps {
    pub title: OrderStatus,
    pub children: Children,
}

#[function_component(OrderList)]
pub fn order_list(props: &OrderListProps) -> Html {
    let column_id = props.title.to_string();
    let button_class = classes!(
        "text-sm",
        "font-bold",
        "px-2",
        "py-3",
        "border-2",
        props.title.border_color(),
        "rounded-lg"
    );
    let button_text_class = classes!(
        "text-lg",
        "font-semibold",
        "text-center",
        "text-nowrap",
        props.title.text_color()
    );
    let column_class = classes!(
        "h-[500px]",
        "overflow-y-scroll",
        "mt-2",
        "rounded-2xl",
        "px-2",
        "py-2",
        "space-y-3",
        "no-scrollbar",
        props.title.theme_color()
    );

    html! {
        <section>
            <div class={button_class}>
                <p class={button_text_class}>
                    {&props.title.display()}
                </p>
            </div>
            <div
                id={column_id}
                class={column_class}>
                {props.children.clone()}
            </div>
        </section>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct OrderCardProps {
    pub order: OrderInvoiceState,
    pub order_note: NostrNote,
    pub on_click: Callback<SubmitEvent>,
}

#[function_component(OrderCard)]
pub fn order_card(props: &OrderCardProps) -> Html {
    let order_popup = use_state(|| false);
    let open_popup = {
        let order_popup = order_popup.clone();
        Callback::from(move |_| order_popup.set(true))
    };
    html! {
        <>
        <OrderStateCard order={props.order.clone()} on_click={open_popup} />
        <PopupSection close_handle={order_popup.clone()}>
            <OrderDetailModal order={props.order.clone()} on_submit={props.on_click.clone()} />
        </PopupSection>
        </>
    }
}
#[derive(Clone, PartialEq, Properties)]
pub struct OrderPickupProps {
    pub order: OrderInvoiceState,
    pub commerce_profile: CommerceProfile,
    pub on_click: Callback<SubmitEvent>,
}

#[function_component(OrderPickup)]
pub fn order_card(props: &OrderPickupProps) -> Html {
    let order_popup = use_state(|| false);
    let OrderPickupProps {
        order,
        on_click,
        commerce_profile,
    } = props.clone();
    let open_popup = {
        let order_popup = order_popup.clone();
        Callback::from(move |_| order_popup.set(true))
    };
    html! {
        <>
        <OrderStateCard order={order.clone()} on_click={open_popup} />
        <PopupSection close_handle={order_popup.clone()}>
            <OrderPickupModal {order} on_order_click={on_click} {commerce_profile} />
        </PopupSection>
        </>
    }
}
#[derive(Clone, PartialEq, Properties)]
pub struct OrderPickupModalProps {
    pub order: OrderInvoiceState,
    pub commerce_profile: CommerceProfile,
    pub on_order_click: Callback<SubmitEvent>,
}
#[function_component(OrderPickupModal)]
pub fn order_detail_modal(props: &OrderPickupModalProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);
    let translations = language_ctx.translations();
    let OrderPickupModalProps {
        order,
        commerce_profile,
        on_order_click,
    } = props;
    let request = order.get_order_request();
    let customer_profile = &request.profile;
    let address: GeolocationCoordinates = request.address.coordinates().into();
    let commerce_address = commerce_profile.geolocation();
    let order_state = order.order_status.clone();
    html! {
        <main class="bg-white rounded-2xl py-5 px-10 mx-auto max-w-xl h-[640px] overflow-y-auto">
            <div class="flex items-center justify-between border-b border-b-gray-400 pb-3">
                <div>
                    <p class="text-fuente-dark font-bold text-2xl">{format!("#{}", &order.order_id()[..12])}</p>
                    <p class="text-gray-500 font-light text-lg">{&translations["store_order_modal_title"]}</p>
                </div>
                <button
                    class="border-2 border-gray-400 text-gray-400 bg-white rounded-2xl py-3 px-4 text-center font-semibold">{order.order_status.to_string()}</button>
            </div>

            <OrderPickupMapPreview
                order_id={order.order_id()}
                commerce_location={commerce_address}
                consumer_location={address}
                own_location={location_state.clone()}
                classes={classes!["rounded-lg", "min-w-64", "min-h-64", "h-64", "w-full", "p-2"]}
            />
            {match order_state {
                OrderStatus::ReadyForDelivery => html! {
                    <div class="grid grid-cols-1 gap-4">
                        <CommerceProfileDetails commerce_data={commerce_profile.clone()} />
                        <CommerceProfileAddressDetails commerce_data={commerce_profile.clone()} />
                        <CustomerDetails customer={customer_profile.clone()} />
                    </div>
                },
                OrderStatus::InDelivery => html! {
                    <div class="grid grid-cols-1 gap-4">
                        <CustomerDetails customer={customer_profile.clone()} />
                        <CustomerAddressDetails customer={request.address.clone()} />
                    </div>
                },
                _ => html! {<></>},
            }}
            <form onsubmit={on_order_click.clone()} class="mt-5">
            <select id="order_status" name="order_status" class="hidden">
                <option value={OrderStatus::ReadyForDelivery.to_string()}></option>
            </select>
            <input type="submit" value={translations["store_order_modal_button_submit"].clone()}
                class="bg-fuente-orange text-white text-center text-lg font-bold rounded-full w-full py-3 mt-5" />
            </form>
        </main>
    }
}
#[derive(Clone, PartialEq, Properties)]
pub struct OrderPickupMapPreviewProps {
    pub order_id: String,
    pub commerce_location: GeolocationCoordinates,
    pub consumer_location: GeolocationCoordinates,
    pub own_location: UseStateHandle<Option<GeolocationCoordinates>>,
    pub classes: Classes,
}
#[function_component(OrderPickupMapPreview)]
pub fn order_pickup_map_preview(props: &OrderPickupMapPreviewProps) -> Html {
    let OrderPickupMapPreviewProps {
        order_id,
        commerce_location,
        consumer_location,
        own_location,
        classes,
    } = props.clone();

    let map_state: UseStateHandle<Option<LeafletMap>> = use_state(|| None);
    let markers: UseStateHandle<Vec<(f64, f64)>> = use_state(|| vec![]);
    let map_id = format!("order-map-{}", order_id);
    let own_marker_state = use_state(|| None::<Marker>);

    let map_options = LeafletMapOptions {
        zoom: 13,
        zoom_control: true,
        scroll_wheel_zoom: true,
        double_click_zoom: true,
        dragging: true,
        min_zoom: Some(3),
        max_zoom: Some(18),
        ..Default::default()
    };

    use_effect_with(map_state.clone(), move |map_state| {
        if let Some(map) = map_state.as_ref() {
            let commerce_icon = IconOptions {
                icon_url: "/public/assets/img/pay_pickup.png".to_string(),
                icon_size: Some(vec![32, 32]),
                icon_anchor: Some(vec![16, 16]),
            };
            let _ = map.add_marker_with_icon(&commerce_location, commerce_icon);
            let consumer_icon = IconOptions {
                icon_url: "/public/assets/img/my_marker.png".to_string(),
                icon_size: Some(vec![32, 32]),
                icon_anchor: Some(vec![16, 16]),
            };
            let _ = map.add_marker_with_icon(&consumer_location, consumer_icon);
            let bounds = vec![
                vec![commerce_location.latitude, commerce_location.longitude],
                vec![consumer_location.latitude, consumer_location.longitude],
            ];
            let js_value_bounds = serde_wasm_bindgen::to_value(&bounds).unwrap();
            let _ = map.fitBounds(&js_value_bounds);
        }
        || {}
    });
    let location_icon_options = Some(IconOptions {
        icon_url: "/public/assets/img/rider2.png".to_string(),
        icon_size: Some(vec![32, 32]),
        icon_anchor: Some(vec![16, 16]),
    });
    html! {
        <LeafletComponent
            {map_id}
            {map_options}
            {location_icon_options}
            markers={(*markers).clone()}
            on_location_changed={Callback::from({
                let location_state = own_location.clone();
                move |coords: GeolocationCoordinates| {
                    location_state.set(Some(coords));
                }
            })}
            on_map_created={Callback::from({
                let map = map_state.clone();
                move |map_instance: LeafletMap| map.set(Some(map_instance))
            })}
            on_marker_created={Callback::from({
                move |marker: Marker| {
                    own_marker_state.set(Some(marker));
                }
            })}
            class={classes}
            style="height: 100%; width: 100%; border-radius: 1rem; border: 2px solid #f0f0f0;"
        />
    }
}
