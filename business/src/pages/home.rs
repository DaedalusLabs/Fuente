use crate::contexts::{CommerceDataStore, OrderDataStore};
use fuente::{
    // js::draggable::Droppable,
    mass::LoadingScreen,
    models::{OrderInvoiceState, OrderStatus, OrderUpdateRequest, NOSTR_KIND_COMMERCE_UPDATE},
};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::notes::NostrNote;
use yew::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>();
    if commerce_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    html! {
        <OrderDashboard />
    }
}

#[function_component(OrderDashboard)]
pub fn order_dashboard() -> Html {
    let commerce_ctx = use_context::<OrderDataStore>().expect("No commerce ctx");
    html! {
        <main class="container mx-auto mt-10 max-h-full pb-4 overflow-y-clip no-scrollbar">
            <div class="flex justify-between items-center">
                <h1 class="text-fuente text-6xl tracking-tighter font-bold">{"My Orders"}</h1>
                <button class="border-2 border-fuente rounded-full py-3 px-10 text-center text-xl text-fuente font-semibold">{"View Historic"}</button>
            </div>

            <div class="flex gap-10 mt-10 min-h-96 h-full">
                <div class="grid grid-cols-2 gap-4 lg:w-1/2 xl:w-[40%] 2xl:w-[30%] h-[calc(100vh-16rem)]">
                    <OrderList title={OrderStatus::Pending} orders={commerce_ctx.filter_by_order_status(OrderStatus::Pending)} />
                    <OrderList title={OrderStatus::Preparing} orders={commerce_ctx.filter_by_order_status(OrderStatus::Preparing)} />
                </div>

                <div class="grid grid-cols-2 gap-4 lg:w-[65%] xl:w-[40%] 2xl:w-[30%] h-[calc(100vh-16rem)]">
                    <OrderList title={OrderStatus::ReadyForDelivery} orders={commerce_ctx.filter_by_order_status(OrderStatus::ReadyForDelivery)} />
                    <OrderList title={OrderStatus::InDelivery} orders={commerce_ctx.filter_by_order_status(OrderStatus::InDelivery)} />
                </div>

                <div class="grid grid-cols-2 gap-4 lg:w-1/2 xl:w-[40%] 2xl:w-[30%] h-[calc(100vh-16rem)]">
                    <OrderList title={OrderStatus::Completed} orders={commerce_ctx.filter_by_order_status(OrderStatus::Completed)} />
                    <OrderList title={OrderStatus::Canceled} orders={commerce_ctx.filter_by_order_status(OrderStatus::Canceled)} />
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
    let theme_color = match props.title {
        OrderStatus::Pending => "bg-gray-100",
        OrderStatus::Preparing => "bg-orange-100",
        OrderStatus::ReadyForDelivery => "bg-sky-100",
        OrderStatus::InDelivery => "bg-orange-100",
        OrderStatus::Completed => "bg-green-100",
        OrderStatus::Canceled => "bg-red-100",
    };
    let text_color = match props.title {
        OrderStatus::Pending => "text-gray-500",
        OrderStatus::Preparing => "text-orange-500",
        OrderStatus::ReadyForDelivery => "text-sky-500",
        OrderStatus::InDelivery => "text-orange-500",
        OrderStatus::Completed => "text-green-500",
        OrderStatus::Canceled => "text-red-500",
    };
    let border_color = match props.title {
        OrderStatus::Pending => "border-gray-500",
        OrderStatus::Preparing => "border-orange-500",
        OrderStatus::ReadyForDelivery => "border-sky-500",
        OrderStatus::InDelivery => "border-orange-500",
        OrderStatus::Completed => "border-green-500",
        OrderStatus::Canceled => "border-red-500",
    };
    let button_class = classes!("text-sm", "font-bold", "px-4", "py-2", "border", border_color, "rounded-lg");
    let button_text_class = classes!("text-lg", "font-semibold", "text-center", "text-nowrap", text_color);
    let column_class = classes!("h-full", "min-w-fit", "max-h-[calc(100vh-16rem)]", "mt-2", "rounded-2xl", "overflow-y-auto", "no-scrollbar", theme_color);

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
    let id = &props.order.order;
    let order = props.order.get_order_request();
    let order_status = &props.order.order_status;
    let customer_name = order.profile.nickname;
    let products = order.products.counted_products();
    let value = order.products.total();
    let accept_order = {
        let user_keys = key_ctx.get_nostr_key().unwrap();
        let send_note = relay_ctx.send_note.clone();
        let order_confirmation = props.order.clone();
        let order = props.order_note.clone();
        Callback::from(move |_: MouseEvent| {
            let status_update = match &order_confirmation.order_status {
                OrderStatus::Pending => OrderStatus::Preparing,
                OrderStatus::Preparing => OrderStatus::ReadyForDelivery,
                _ => return,
            };
            let new_request = OrderUpdateRequest {
                order: order.clone(),
                status_update,
            };
            let signed_req = new_request
                .sign_update(&user_keys, NOSTR_KIND_COMMERCE_UPDATE)
                .expect("Could not sign order");
            send_note.emit(signed_req);
        })
    };
    let cancel_order = {
        let user_keys = key_ctx.get_nostr_key().unwrap();
        let send_note = relay_ctx.send_note.clone();
        let order = props.order_note.clone();
        Callback::from(move |_| {
            let status_update = OrderStatus::Canceled;
            let new_request = OrderUpdateRequest {
                order: order.clone(),
                status_update,
            };
            let signed_req = new_request
                .sign_update(&user_keys, NOSTR_KIND_COMMERCE_UPDATE)
                .expect("Could not sign order");
            send_note.emit(signed_req);
        })
    };
    html! {
        <div
            id={id.id.as_ref().unwrap().to_string()}
            class="draggable h-fit col-span-1 bg-white rounded-lg shadow-lg m-1 p-4">
            <div class="flex flex-row justify-between select-none">
                <div class="flex flex-col gap-2">
                    <span class="text-lg font-bold">{customer_name}</span>
                    {products.iter().map(|product| {
                        html! {
                            <div class="flex flex-row gap-2">
                                <span class="text-sm">{product.0.name()}</span>
                                <span class="text-sm">{product.1}</span>
                            </div>
                        }
                    }).collect::<Html>()}
                </div>
                <div class="flex flex-col gap-2">
                    <span class="text-lg font-bold">{value}</span>
                    <span class="text-sm">{order_status.to_string()}</span>
                </div>
            </div>
            <div class="flex flex-row gap-4 select-none">
                {match order_status {
                    OrderStatus::Pending => html! {
                        <>
                        <button
                            onmousedown={accept_order}
                            class="text-sm font-bold px-4 py-2 border border-purple-900 rounded-lg">{"Accept"}</button>
                        <button
                            onclick={cancel_order}
                            onmousedown={|event: MouseEvent| event.stop_propagation()}
                            class="text-sm font-bold px-4 py-2 border border-red-500 rounded-lg">{"Decline"}</button>
                        </>
                    },
                    OrderStatus::Preparing => html! {
                        <>
                        <button
                            onmousedown={accept_order}
                            class="text-sm font-bold px-4 py-2 border border-purple-900 rounded-lg">{"Ready"}</button>
                        </>
                    },
                    _ => html! {}
                }}
            </div>
        </div>
    }
}
