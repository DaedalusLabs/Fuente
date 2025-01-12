use crate::{
    contexts::{CommerceDataStore, OrderDataStore},
    router::CommerceRoute,
};
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{LoadingScreen, OrderCard, OrderList},
    models::{OrderStatus, OrderUpdateRequest, NOSTR_KIND_COMMERCE_UPDATE},
};
use lucide_yew::{Check, CircleCheck, CircleHelp, Clock2, ScrollText, Truck, X};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>();
    let languague_ctx = use_context::<LanguageConfigsStore>().expect("No language ctx");
    let translations = languague_ctx.translations();
    if commerce_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    let go_to_orders = {
        let router = use_navigator().expect("No router found");
        Callback::from(move |_| {
            router.push(&CommerceRoute::History);
        })
    };
    html! {
        <main class="flex-1 overflow-hidden">
            <div class="flex flex-col h-full">
                <div class="flex flex-row justify-between items-center p-4 lg:p-10">
                    <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-0 lg:text-6xl tracking-tighter font-bold">
                        {&translations["orders_heading"]}
                    </h1>
                    <button onclick={go_to_orders.clone()}
                        class="block lg:hidden flex items-center bg-fuente-buttons p-2 rounded-xl">
                            <ScrollText class="w-6 h-6 text-fuente" />
                    </button>
                    <button onclick={go_to_orders}
                        class="lg:block hidden flex items-center bg-fuente-buttons px-6 py-3 rounded-full text-fuente-forms space-x-2 font-bold text-sm md:text-md lg:text-lg">
                        {&translations["orders_historic"]}
                    </button>
                </div>
                <OrderDashboardMobile />
                <OrderDashboardDesktop />
            </div>
        </main>
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

#[derive(Clone, PartialEq, Properties)]
pub struct MobileDashboardOptionsProps {
    pub order_status: UseStateHandle<OrderStatus>,
}

#[function_component(MobileDashboardOptions)]
pub fn order_dashboard(props: &MobileDashboardOptionsProps) -> Html {
    fn set_viewed_status(
        status: OrderStatus,
        set_status: &UseStateHandle<OrderStatus>,
    ) -> Callback<MouseEvent> {
        let handle = set_status.clone();
        Callback::from(move |_| {
            handle.set(status.clone());
        })
    }
    let button_classes = classes!(
        "rounded-lg",
        "md:rounded-xl",
        "p-2",
        "h-fit",
        "flex",
        "justify-center",
        "items-center"
    );
    html! {
        <div class="grid grid-flow-col items-center gap-2 px-4 justify-stretch">
            <button
                onclick={set_viewed_status(OrderStatus::Pending, &props.order_status)}
                class={
                    let mut new_class = button_classes.clone();
                    new_class.push("bg-gray-500");
                    new_class
                }>
                <CircleHelp class="w-6 h-6 md:w-8 md:h-8 text-white" />
            </button>

            <button
                onclick={set_viewed_status(OrderStatus::Preparing, &props.order_status)}
                class={
                    let mut new_class = button_classes.clone();
                    new_class.push("bg-orange-500");
                    new_class
                }>
                <Clock2 class="w-6 h-6 md:w-8 md:h-8 text-white" />
            </button>

            <button
                onclick={set_viewed_status(OrderStatus::ReadyForDelivery, &props.order_status)}
                class={
                    let mut new_class = button_classes.clone();
                    new_class.push("bg-sky-500");
                    new_class
                }>
                <CircleCheck class="w-6 h-6 md:w-8 md:h-8 text-white" />
            </button>

            <div
                onclick={set_viewed_status(OrderStatus::InDelivery, &props.order_status)}
                class={
                    let mut new_class = button_classes.clone();
                    new_class.push("bg-orange-500");
                    new_class
                }>
                <Truck class="w-6 h-6 md:w-8 md:h-8 text-white" />
            </div>

            <button
                onclick={set_viewed_status(OrderStatus::Completed, &props.order_status)}
                class={
                    let mut new_class = button_classes.clone();
                    new_class.push("bg-green-500");
                    new_class
                }>
                <Check class="w-6 h-6 md:w-8 md:h-8 text-white" />
            </button>

            <button
                onclick={set_viewed_status(OrderStatus::Canceled, &props.order_status)}
                class={
                    let mut new_class = button_classes.clone();
                    new_class.push("bg-red-500");
                    new_class
                }>
                <X class="w-6 h-6 md:w-8 md:h-8 text-white" />
            </button>
        </div>
    }
}
#[function_component(OrderDashboardMobile)]
pub fn order_dashboard() -> Html {
    let commerce_ctx = use_context::<OrderDataStore>().expect("No commerce ctx");
    let send_note = use_context::<NostrProps>().expect("Nostr context not found");
    let update_kind = NOSTR_KIND_COMMERCE_UPDATE;
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let nostr_keys = key_ctx.get_nostr_key().expect("Nostr key not found");
    let viewed_status = use_state(|| OrderStatus::Pending);
    html! {
        <div class="lg:hidden flex flex-col flex-1 overflow-hidden">
            <MobileDashboardOptions order_status={viewed_status.clone()} />
            <div class="flex-1 overflow-hidden mt-4 px-4">
                <OrderList title={*viewed_status}>
                    {commerce_ctx.filter_by_order_status(*viewed_status).iter().map(|order| {
                        let on_click = {
                            respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                        };
                        html! {
                            <OrderCard order={order.0.clone()} on_click={on_click} order_note={order.1.clone()} />
                        }
                    }).collect::<Html>()}
                </OrderList>
            </div>
        </div>
    }
}
#[function_component(OrderDashboardDesktop)]
pub fn order_dashboard() -> Html {
    let commerce_ctx = use_context::<OrderDataStore>().expect("No commerce ctx");
    let send_note = use_context::<NostrProps>().expect("Nostr context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let update_kind = NOSTR_KIND_COMMERCE_UPDATE;
    let nostr_keys = key_ctx.get_nostr_key().expect("Nostr key not found");
    html! {
        <div class="hidden lg:flex flex-1 overflow-hidden">
            <div class="flex justify-start gap-4 h-full p-4 overflow-x-auto">
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
            </div>
        </div>
    }
}
