use lucide_yew::ArrowLeft;
use yew::prelude::*;
use crate::mass::OrderStateCard;

use crate::{contexts::LanguageConfigsStore, models::{OrderInvoiceState, OrderStatus}};


#[derive(Properties, Clone, PartialEq)]
pub struct OrderHistoryProps {
    pub completed_orders: Vec<OrderInvoiceState>,
    pub canceled_orders: Vec<OrderInvoiceState>,
    pub on_order_click: Callback<MouseEvent>,
}

#[function_component(OrderHistoryMobile)]
pub fn order_history_desktop(props: &OrderHistoryProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let filter = use_state(|| OrderStatus::Completed);
    let OrderHistoryProps {
        completed_orders,
        canceled_orders,
        on_order_click,
    } = props;
    let bg_color = match *filter {
        OrderStatus::Completed => "bg-green-100",
        OrderStatus::Canceled => "bg-red-100",
        _ => "bg-gray-100",
    };
    html! {
        <div class="flex lg:hidden flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                <div class="flex flex-col gap-2 w-full h-full overflow-hidden">
                    <div class="grid grid-flow-col justify-stretch gap-2 w-full">
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(OrderStatus::Completed)
                            })}
                            class={classes!("border-green-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                            <p class={classes!("text-lg", "font-semibold", "text-center", "text-green-500")}>
                                {&translations["store_orders_history_completed"]}
                            </p>
                        </div>
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(OrderStatus::Canceled)
                            })}
                            class={classes!("border-red-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                            <p class={classes!("text-lg", "font-semibold", "text-center", "text-red-500")}>
                                {&translations["store_orders_history_canceled"]}
                            </p>
                        </div>
                    </div>
                    <div
                        class={classes!("flex-1", "rounded-2xl", "mt-2", "px-2", "py-2", "overflow-y-auto", "no-scrollbar", bg_color)}>
                        <div class="grid grid-cols-1 gap-4">
                        {
                            match *filter {
                                OrderStatus::Completed => completed_orders.iter().map(|order| {
                                    html! { <OrderStateCard order={(*order).clone()} on_click={on_order_click.clone()} />}
                                }).collect::<Html>(),
                                OrderStatus::Canceled => canceled_orders.iter().map(|order| {
                                    html! { <OrderStateCard order={(*order).clone()} on_click={Callback::noop()} />}
                                }).collect::<Html>(),
                                _ => html! {},
                            }
                        }
                        </div>
                    </div>

                </div>
            </div>
        </div>
    }
}

#[function_component(OrderHistoryDesktop)]
pub fn order_history_desktop(props: &OrderHistoryProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let OrderHistoryProps {
        completed_orders,
        canceled_orders,
        on_order_click: _,
    } = props;
    html! {
        <div class="hidden lg:flex flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                <div class="flex flex-col gap-2 w-1/2 h-full overflow-hidden">
                    <div class="border-2 border-green-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-green-500 text-lg font-semibold text-center">{&translations["store_orders_history_completed"]}</p>
                    </div>

                     <div class={"flex-1 rounded-2xl mt-2 px-2 py-2 overflow-y-auto no-scrollbar bg-green-100"}>
                        <div class="grid grid-cols-1 gap-4">
                        {completed_orders.iter().map(|order| {
                           html! {  <OrderStateCard order={(*order).clone()} on_click={Callback::noop()} />}
                        }).collect::<Html>()}
                        </div>
                    </div>

                </div>

                <div class="flex flex-col gap-2 w-1/2 h-full overflow-hidden">
                    <div class="border-2 border-red-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-red-500 text-lg font-semibold text-center">{&translations["store_orders_history_canceled"]}</p>
                    </div>

                     <div class={"flex-1 rounded-2xl mt-2 px-2 py-2 overflow-y-auto no-scrollbar bg-red-100"}>
                        <div class="grid grid-cols-1 gap-4">
                        {canceled_orders.iter().map(|order| {
                            html! { <OrderStateCard order={(*order).clone()} on_click={Callback::noop()} />}
                        }).collect::<Html>()}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct OrderDetailsProps {
    pub order: OrderInvoiceState,
    pub on_back: Callback<MouseEvent>,
}

#[function_component(OrderDetails)]
pub fn order_details(props: &OrderDetailsProps) -> Html {
    let order_req = props.order.get_order_request();
    let products = order_req.products.counted_products();

    html! {
        <div class="flex flex-col w-full h-full p-4 max-w-4xl mx-auto">
          <div class="flex items-center gap-4 mb-6">
            <button
                onclick={props.on_back.clone()}
              class="p-2 rounded-lg hover:bg-gray-100 transition-colors duration-200"
            >
              <ArrowLeft class="w-5 h-5" />
            </button>
            <h2 class="text-2xl font-semibold text-fuente-dark">
                {format!("Order Details #{}", props.order.order_id()[..8].to_string())}
            </h2>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div class="space-y-6">
             <div class="space-y-2">
                <h3 class="font-medium text-fuente">{"Customer Information"}</h3>
                <p>{format!("Name: {}", order_req.profile.nickname)}</p>
                <p>{format!("Phone: {}", order_req.profile.telephone)}</p>
                <p>{format!("Email: {}", order_req.profile.email)}</p>
              </div>
            </div>
            <div class="space-y-2">
                <h3 class="font-medium text-fuente">{"Delivery Address"}</h3>
                <p class="text-sm">{order_req.address.lookup().display_name()}</p>
            </div>
            <div class="space-y-2">
                <h3 class="font-medium text-fuente">{"Order Status"}</h3>
                <p class={classes!(
                    "font-medium",
                    if props.order.order_status == OrderStatus::Completed {
                        "text-green-600"
                    } else {
                        "text-red-600"
                    }
                )}>
                    {props.order.order_status.display()}
                </p>
            </div>

                <div class="space-y-4">
                    <h3 class="font-medium text-fuente">{"Order Items"}</h3>
                    <div class="space-y-2">
                        {products.iter().map(|(product, count)| {
                            let subtotal = product.price().parse::<f64>().unwrap() * *count as f64;
                            html! {
                                <div class="flex justify-between py-2 border-b">
                                    <div>
                                        <p class="font-medium">{product.name()}</p>
                                        <p class="text-sm text-gray-500">
                                            {format!("{} x {} SRD", count, product.price())}
                                        </p>
                                    </div>
                                    <p class="font-medium">{format!("{:.2} SRD", subtotal)}</p>
                                </div>
                            }
                        }).collect::<Html>()}

                        <div class="flex justify-between pt-4 font-medium">
                            <p>{"Total"}</p>
                            <p>{format!("{:.2} SRD", order_req.products.total())}</p>
                        </div>
                    </div>
                </div>
          </div>
        </div>
    }
}
