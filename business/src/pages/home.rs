use crate::contexts::{CommerceDataStore, OrderDataStore};
use fuente::{
    // js::draggable::Droppable,
    mass::LoadingScreen,
    models::{OrderInvoiceState, OrderStatus},
};
use minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
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
    // use_effect_with((), |_| {
    //     let droppable = Droppable::init(".draggable-zone", ".draggable", ".draggable-col").unwrap();
    //     droppable.event_listener();
    //     || {}
    // });
    let commerce_ctx = use_context::<OrderDataStore>().expect("No commerce ctx");
    html! {
        <div class="draggable-zone flex flex-row gap-8">
            <OrderList title={OrderStatus::Pending} orders={commerce_ctx.filter_by_order_status(OrderStatus::Pending)} />
            <OrderList title={OrderStatus::Preparing} orders={commerce_ctx.filter_by_order_status(OrderStatus::Preparing)} />
            <OrderList title={OrderStatus::ReadyForDelivery} orders={commerce_ctx.filter_by_order_status(OrderStatus::ReadyForDelivery)} />
            <OrderList title={OrderStatus::InDelivery} orders={commerce_ctx.filter_by_order_status(OrderStatus::InDelivery)} />
            <OrderList title={OrderStatus::Completed} orders={commerce_ctx.filter_by_order_status(OrderStatus::Completed)} />
            <OrderList title={OrderStatus::Canceled} orders={commerce_ctx.filter_by_order_status(OrderStatus::Canceled)} />
        </div>
    }
}
#[derive(Clone, PartialEq, Properties)]
pub struct OrderListProps {
    pub title: OrderStatus,
    pub orders: Vec<OrderInvoiceState>,
}

#[function_component(OrderList)]
pub fn order_list(props: &OrderListProps) -> Html {
    let column_id = props.title.to_string();
    html! {
        <div class="flex flex-col min-w-64">
            <h2 class="text-2xl text-nowrap">{&props.title.display()}</h2>
            <div
                id={column_id}
                class="draggable-col h-full flex flex-col gap-4 overflow-y-scroll no-scrollbar p-2">
                {props.orders.iter().map(|order| {
                    html! {
                        <OrderCard order={order.clone()} />
                    }
                }).collect::<Html>()}
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct OrderCardProps {
    pub order: OrderInvoiceState,
}

#[function_component(OrderCard)]
pub fn order_card(props: &OrderCardProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No user context found");
    let relay_ctx = use_context::<NostrProps>().expect("No relay context found");
    let id = props.order.get_order();
    let order = props.order.get_order_request();
    let order_status = props.order.get_order_status();
    let customer_name = order.profile.nickname();
    let products = order.products.counted_products();
    let value = order.products.total();
    let accept_order = {
        let user_keys = key_ctx.get_nostr_key().unwrap();
        let send_note = relay_ctx.send_note.clone();
        let order_confirmation = props.order.clone();
        Callback::from(move |_: MouseEvent| {
            let mut order_confirmation = order_confirmation.clone();
            let old_status = order_confirmation.get_order_status();
            match old_status {
                OrderStatus::Pending => {
                    order_confirmation.update_order_status(OrderStatus::Preparing)
                }
                OrderStatus::Preparing => {
                    order_confirmation.update_order_status(OrderStatus::ReadyForDelivery)
                }
                _ => {}
            }
            let signed_confirmation = order_confirmation
                .sign_server_request(&user_keys)
                .expect("Failed to sign order confirmation");
            send_note.emit(signed_confirmation);
        })
    };
    let cancel_order = {
        let user_keys = key_ctx.get_nostr_key().unwrap();
        let send_note = relay_ctx.send_note.clone();
        let order_confirmation = props.order.clone();
        Callback::from(move |_| {
            let mut order_confirmation = order_confirmation.clone();
            order_confirmation.update_order_status(OrderStatus::Canceled);
            let signed_confirmation = order_confirmation
                .sign_server_request(&user_keys)
                .expect("Failed to sign order confirmation");
            send_note.emit(signed_confirmation);
        })
    };
    html! {
        <div
            id={id.get_id().to_string()}
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
