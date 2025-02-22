use crate::contexts::{CommerceDataStore, LiveOrderStore};
use crate::router::ConsumerRoute;
use fuente::models::OrderPaymentStatus;
use fuente::{
    mass::{OrderDetailModal, OrderPickupMapPreview, OrderStateCard, PopupSection},
    models::{OrderInvoiceState, OrderStatus},
};
use lucide_yew::Package;
use nostr_minions::browser_api::GeolocationCoordinates;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(PackageListMobile)]
pub fn package_list_mobile(props: &html::ChildrenProps) -> Html {
    html! {
        <div class="flex lg:hidden flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full overflow-hidden">
                <div class="flex flex-col gap-2 w-full h-full overflow-hidden">
                    <div
                        class={classes!("flex-1", "rounded-2xl", "mt-2", "px-2", "py-2", "overflow-y-auto", "no-scrollbar", "bg-yellow-100")}>
                        <div class="grid grid-cols-1 gap-4">
                            {props.children.clone()}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
#[function_component(PackageListDesktop)]
pub fn package_list_mobile(props: &html::ChildrenProps) -> Html {
    html! {
        <div class="hidden lg:flex flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full overflow-hidden">
                 <div class={"flex-1 rounded-2xl px-2 py-2 overflow-y-auto no-scrollbar bg-yellow-100"}>
                    <div class="grid grid-cols-1 gap-4">
                        {props.children.clone()}
                    </div>
                </div>
            </div>
        </div>
    }
}
#[function_component(TrackPackagesPage)]
pub fn track_packages_page() -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    let selected_order = use_state(|| None::<OrderInvoiceState>);
    let order_popup = use_state(|| false);
    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);

    // Get live orders that are in progress
    let active_orders = order_ctx
        .live_orders
        .iter()
        .filter(|(_, state)| {
            state.order_status != OrderStatus::Completed
                && state.order_status != OrderStatus::Canceled
                && state.payment_status == OrderPaymentStatus::PaymentSuccess
        })
        .map(|(_, order)| order.clone())
        .collect::<Vec<OrderInvoiceState>>();

    if active_orders.is_empty() {
        return html! {
            <div class="flex flex-col items-center justify-center h-full p-8">
                <Package class="w-16 h-16 text-gray-300 mb-4" />
                <h2 class="text-xl font-semibold text-gray-600 mb-2">{"No Active Orders"}</h2>
                <p class="text-gray-500 text-center">{"You don't have any packages being delivered right now"}</p>
                <Link<ConsumerRoute>
                    to={ConsumerRoute::Home}
                    classes="mt-6 px-6 py-2 bg-fuente text-white rounded-lg hover:bg-fuente-dark"
                >
                    {"Start Shopping"}
                </Link<ConsumerRoute>>
            </div>
        };
    }
    let orders = {
        active_orders
            .into_iter()
            .map(|order| {
                let onclick = {
                    let order_clone = order.clone();
                    let order_popup = order_popup.clone();
                    let selected_order = selected_order.clone();
                    Callback::from(move |_| {
                        selected_order.set(Some(order_clone.clone()));
                        order_popup.set(true);
                    })
                };

                html! {
                    <>
                        // Order Card Component
                        <OrderStateCard order={order.clone()} on_click={onclick.clone()} />
                         // Modal with Map and Details
                    </>
                }
            })
            .collect::<Html>()
    };

    html! {
        <main class="flex flex-col h-full overflow-hidden container mx-auto">
            <div class="flex flex-row justify-center lg:justify-between items-center py-4 lg:py-0 lg:pb-10">
                <h1 class="text-fuente font-mplus text-3xl text-center lg:text-left lg:text-6xl tracking-tighter font-bold">
                    {"Track Your Packages"}
                </h1>
            </div>
            <div class="flex-1 flex flex-col lg:flex-row">
                <PackageListMobile>
                    {orders.clone()}
                </PackageListMobile>
                <PackageListDesktop>
                    {orders}
                </PackageListDesktop>
                {if let Some(order) = (*selected_order).clone() {
                    let has_courier = order.courier.is_some();
                    let order_id = order.order_id();
                    let order_req = order.get_order_request();
                    let commerce = commerce_ctx.find_commerce_by_id(&order_req.commerce)
                        .expect("Commerce not found");
                    html! {
                        <PopupSection close_handle={order_popup.clone()}>
                            <div class="bg-white rounded-2xl p-6 max-w-xs sm:max-w-sm md:max-w-md lg:mx-w-4xl max-h-[85vh] m-4 overflow-y-auto no-scrollbar">
                                <OrderDetailModal
                                    order={order.clone()}
                                    on_submit={Callback::from(move |_| {})}
                                />

                                // Map Section if driver is assigned
                                {if has_courier {
                                    let commerce_address = commerce.profile().geolocation();
                                    let delivery_location: GeolocationCoordinates = order_req.address.coordinates().into();

                                    html! {
                                        <div class="mt-6">
                                            <h3 class="text-xl font-bold mb-4">{"Live Tracking"}</h3>
                                            <OrderPickupMapPreview
                                                order_id={order_id.clone()}
                                                commerce_location={commerce_address}
                                                consumer_location={delivery_location}
                                                own_location={location_state.clone()}
                                                classes={classes!["h-96", "w-full", "rounded-lg"]}
                                            />
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </PopupSection>
                        }
                } else {
                    html! {}
                }}
            </div>
        </main>
    }
}
