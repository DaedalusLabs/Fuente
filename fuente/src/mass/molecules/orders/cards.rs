use crate::mass::{OrderDetailModal, OrderPickupModal, PopupSection};
use nostro2::notes::NostrNote;
use yew::prelude::*;

use crate::models::{CommerceProfile, OrderInvoiceState};

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
        <div onclick={props.on_click.clone()} id={order_id} class="bg-white shadow py-2 px-5 rounded-2xl space-y-1 cursor-pointer mt-3">
            <p class="pointer-events-none text-fuente font-bold text-md">{profile.nickname}</p>
            <p class="pointer-events-none font-bold text-sm">{format!("#{}", &order.order_id()[..8])}</p>
            <p class="pointer-events-none text-gray-500 text-xs">{format!("{} | {}", locale_date, locale_time)}</p>
        </div>
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
            <main 
                class="bg-white rounded-2xl p-4 max-h-screen m-4 overflow-y-auto scrollbar-none md:w-1/2 mx-auto pointer-events-auto">
                <OrderDetailModal order={props.order.clone()} on_submit={props.on_click.clone()} />
            </main>
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
