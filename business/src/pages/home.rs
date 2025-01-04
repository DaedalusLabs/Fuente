use crate::{
    contexts::{CommerceDataStore, OrderDataStore},
    router::CommerceRoute,
};
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{LoadingScreen, OrderDetailModal, OrderStateCard, PopupSection},
    models::{OrderInvoiceState, OrderStatus, OrderUpdateRequest, NOSTR_KIND_COMMERCE_UPDATE},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::notes::NostrNote;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>();
    if commerce_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    html! {
        <>
        <OrderDashboard />
        </>

    }
}

#[function_component(OrderDashboard)]
pub fn order_dashboard() -> Html {
    let commerce_ctx = use_context::<OrderDataStore>().expect("No commerce ctx");
    let languague_ctx = use_context::<LanguageConfigsStore>().expect("No language ctx");
    let translations = languague_ctx.translations();
    let go_to_orders = {
        let router = use_navigator().expect("No router found");
        Callback::from(move |_| {
            router.push(&CommerceRoute::History);
        })
    };
    html! {
        <main class="container mx-auto mt-10">
            <div class="flex justify-between items-center">
                <h1 class="text-fuente text-6xl tracking-tighter font-bold">{&translations["orders_heading"]}</h1>
                <button onclick={go_to_orders}
                    class="border-2 border-fuente rounded-full py-3 px-10 text-center text-xl text-fuente font-semibold">
                    {&translations["orders_historic"]}
                </button>
            </div>

            <div class="flex justify-center gap-10 mt-10 min-h-96">
                <div class="grid grid-cols-2 gap-2 lg:w-1/2 xl:w-[40%] 2xl:w-[30%]">
                    <section>
                        <OrderList title={OrderStatus::Pending} orders={commerce_ctx.filter_by_order_status(OrderStatus::Pending)} />
                    </section>

                    <section>
                        <OrderList title={OrderStatus::Preparing} orders={commerce_ctx.filter_by_order_status(OrderStatus::Preparing)} />
                    </section>
                </div>

                <div class="grid grid-cols-2 gap-2 lg:w-[65%] xl:w-[40%] 2xl:w-[30%]">
                    <section>
                        <OrderList title={OrderStatus::ReadyForDelivery} orders={commerce_ctx.filter_by_order_status(OrderStatus::ReadyForDelivery)} />
                    </section>

                    <section>
                        <OrderList title={OrderStatus::InDelivery} orders={commerce_ctx.filter_by_order_status(OrderStatus::InDelivery)} />
                    </section>
                </div>

                <div class="grid grid-cols-2 gap-2 lg:w-1/2 xl:w-[40%] 2xl:w-[30%]">
                    <section>
                        <OrderList title={OrderStatus::Completed} orders={commerce_ctx.filter_by_order_status(OrderStatus::Completed)} />
                    </section>

                    <section>
                        <OrderList title={OrderStatus::Canceled} orders={commerce_ctx.filter_by_order_status(OrderStatus::Canceled)} />
                    </section>
                </div>
            </div>
        </main>
    }
}
#[derive(Clone, PartialEq, Properties)]
pub struct OrderListProps {
    pub title: OrderStatus,
    pub orders: Vec<(OrderInvoiceState, NostrNote)>,
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
                {props.orders.iter().map(|order| {
                    html! {
                        <OrderCard order={order.0.clone()} order_note={order.1.clone()} />
                    }
                }).collect::<Html>()}
            </div>
        </section>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct OrderCardProps {
    pub order: OrderInvoiceState,
    pub order_note: NostrNote,
}

#[function_component(OrderCard)]
pub fn order_card(props: &OrderCardProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No user context found");
    let relay_ctx = use_context::<NostrProps>().expect("No relay context found");
    let order_popup = use_state(|| false);
    let respond_to_order = {
        let user_keys = key_ctx.get_nostr_key().unwrap();
        let send_note = relay_ctx.send_note.clone();
        let order = props.order_note.clone();
        let order_popup = order_popup.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let form = HtmlForm::new(e).expect("Could not get form");

            let status_update_str = form
                .select_value("order_status")
                .expect("Could not parse order status");
            let status_update =
                OrderStatus::try_from(status_update_str).expect("Could not parse order status");
            let new_request = OrderUpdateRequest {
                order: order.clone(),
                status_update,
            };
            let signed_req = new_request
                .sign_update(&user_keys, NOSTR_KIND_COMMERCE_UPDATE)
                .expect("Could not sign order");
            send_note.emit(signed_req);
            order_popup.set(false);
        })
    };
    let open_popup = {
        let order_popup = order_popup.clone();
        Callback::from(move |_| order_popup.set(true))
    };
    html! {
        <>
        <OrderStateCard order={props.order.clone()} on_click={open_popup} />
        <PopupSection close_handle={order_popup.clone()}>
            <OrderDetailModal order={props.order.clone()} on_submit={respond_to_order} />
        </PopupSection>
        </>
    }
}
