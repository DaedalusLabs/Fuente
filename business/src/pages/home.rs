use crate::{
    contexts::{CommerceDataStore, OrderDataStore},
    router::CommerceRoute,
};
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{LoadingScreen, OrderCard, OrderList},
    models::{OrderStatus, OrderUpdateRequest, NOSTR_KIND_COMMERCE_UPDATE},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
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

fn respond_to_order(
    nostr_keys: NostrKeypair,
    send_note: Callback<NostrNote>,
    order: NostrNote,
    update_kind: u32,
) -> Callback<SubmitEvent> {
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
            .sign_update(&nostr_keys, update_kind)
            .expect("Could not sign order");
        send_note.emit(signed_req);
    })
}
#[function_component(OrderDashboard)]
pub fn order_dashboard() -> Html {
    let commerce_ctx = use_context::<OrderDataStore>().expect("No commerce ctx");
    let languague_ctx = use_context::<LanguageConfigsStore>().expect("No language ctx");
    let send_note = use_context::<NostrProps>().expect("Nostr context not found");
    let update_kind = NOSTR_KIND_COMMERCE_UPDATE;
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let nostr_keys = key_ctx.get_nostr_key().expect("Nostr key not found");
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
                        <OrderList title={OrderStatus::Pending}>
                            {commerce_ctx.filter_by_order_status(OrderStatus::Pending).iter().map(|order| {
                                let on_click = {
                                    respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                                };
                                html! {
                                    <OrderCard order={order.0.clone()} on_click={on_click} order_note={order.1.clone()} />
                                }
                            }).collect::<Html>()}
                        </OrderList>
                    </section>

                    <section>
                        <OrderList title={OrderStatus::Preparing}>
                            {commerce_ctx.filter_by_order_status(OrderStatus::Preparing).iter().map(|order| {
                                let on_click = {
                                    respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                                };
                                html! {
                                    <OrderCard order={order.0.clone()} on_click={on_click} order_note={order.1.clone()} />
                                }
                            }).collect::<Html>()}
                        </OrderList>
                    </section>
                </div>

                <div class="grid grid-cols-2 gap-2 lg:w-[65%] xl:w-[40%] 2xl:w-[30%]">
                    <section>
                        <OrderList title={OrderStatus::ReadyForDelivery}>
                            {commerce_ctx.filter_by_order_status(OrderStatus::ReadyForDelivery).iter().map(|order| {
                                let on_click = {
                                    respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                                };
                                html! {
                                    <OrderCard order={order.0.clone()} on_click={on_click} order_note={order.1.clone()} />
                                }
                            }).collect::<Html>()}
                        </OrderList>
                    </section>

                    <section>
                        <OrderList title={OrderStatus::InDelivery}>
                            {commerce_ctx.filter_by_order_status(OrderStatus::InDelivery).iter().map(|order| {
                                let on_click = {
                                    respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                                };
                                html! {
                                    <OrderCard order={order.0.clone()} on_click={on_click} order_note={order.1.clone()} />
                                }
                            }).collect::<Html>()}
                        </OrderList>
                    </section>
                </div>

                <div class="grid grid-cols-2 gap-2 lg:w-1/2 xl:w-[40%] 2xl:w-[30%]">
                    <section>
                        <OrderList title={OrderStatus::Completed}>
                            {commerce_ctx.filter_by_order_status(OrderStatus::Completed).iter().map(|order| {
                                let on_click = {
                                    respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                                };
                                html! {
                                    <OrderCard order={order.0.clone()} on_click={on_click} order_note={order.1.clone()} />
                                }
                            }).collect::<Html>()}
                        </OrderList>
                    </section>

                    <section>
                        <OrderList title={OrderStatus::Canceled}>
                            {commerce_ctx.filter_by_order_status(OrderStatus::Canceled).iter().map(|order| {
                                let on_click = {
                                    respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                                };
                                html! {
                                    <OrderCard order={order.0.clone()} on_click={on_click} order_note={order.1.clone()} />
                                }
                            }).collect::<Html>()}
                        </OrderList>
                    </section>
                </div>
            </div>
        </main>
    }
}
