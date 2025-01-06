use crate::contexts::{CommerceDataStore, LiveOrderStore};
use crate::router::ConsumerRoute;
use fuente::{
    mass::{OrderStateCard, OrderDetailModal, OrderPickupMapPreview, PopupSection},
    models::{OrderInvoiceState, OrderStatus},
};
use lucide_yew::{MapPin, Package};
use web_sys::js_sys;
use yew::prelude::*;
use yew_router::prelude::*;
use nostr_minions::browser_api::GeolocationCoordinates;

#[function_component(TrackPackagesPage)]
pub fn track_packages_page() -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    let selected_order = use_state(|| None::<String>);
    let order_popup = use_state(|| false);
    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);

    // Get live orders that are in progress
    let active_orders = order_ctx
        .live_orders
        .iter()
        .filter(|(_, state)| {
            state.order_status != OrderStatus::Completed
                && state.order_status != OrderStatus::Canceled
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

    html! {
        <div class="container mx-auto p-6">
            <h2 class="text-fuente text-4xl pb-10 lg:pb-0 text-center lg:text-left lg:text-6xl font-bold tracking-tighter"> 
                {"Track Your Packages"}
            </h2>
            <div class="grid gap-6">
                {active_orders.into_iter().map(|order| {
                    let order_req = order.get_order_request();
                    let commerce = commerce_ctx.find_commerce_by_id(&order_req.commerce)
                        .expect("Commerce not found");
                    
                    let onclick = {
                        let order_id = order.order_id();
                        let order_popup = order_popup.clone();
                        let selected_order = selected_order.clone();
                        Callback::from(move |_| {
                            selected_order.set(Some(order_id.clone()));
                            order_popup.set(true);
                        })
                    };

                    // Format timestamp from order
                    let date = js_sys::Date::new_0();
                    date.set_time(order.order_timestamp() as f64 * 1000.0);
                    let date_string = date.to_string();

                    let has_courier = order.courier.is_some();
                    let order_id = order.order_id();

                    html! {
                        <div class="bg-white rounded-lg shadow-md p-6">
                            // Order Card Component
                            <OrderStateCard order={order.clone()} on_click={onclick.clone()} />

                            // Additional Info Section
                            <div class="mt-4">
                                // Store Info
                                <div class="flex items-center gap-4 mb-4 p-4 bg-gray-50 rounded-lg">
                                    <img 
                                        src={commerce.profile().logo_url.clone()} 
                                        alt="Store logo" 
                                        class="w-16 h-16 object-cover rounded-lg"
                                    />
                                    <div>
                                        <h4 class="font-semibold text-fuente-dark">{commerce.profile().name.clone()}</h4>
                                        <p class="text-sm text-gray-600">{format!("Purchased on {}", date_string)}</p>
                                    </div>
                                </div>

                                // Driver Info if assigned
                                {if has_courier {
                                    html! {
                                        <div class="flex items-center justify-between mt-4 pt-4 border-t">
                                            <div class="flex items-center gap-2">
                                                <MapPin class="w-5 h-5 text-fuente" />
                                                <span class="font-medium text-fuente">{"Driver Assigned"}</span>
                                            </div>
                                            <span class="text-sm text-blue-600 font-medium">
                                                {"Click to track delivery"}
                                            </span>
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <div class="flex items-center text-gray-600 mt-4 pt-4 border-t">
                                            <Package class="w-5 h-5 mr-2" />
                                            <span>{"Order being prepared"}</span>
                                        </div>
                                    }
                                }}
                            </div>

                            // Modal with Map and Details
                            if let Some(selected_id) = (*selected_order).clone() {
                                if selected_id == order_id {
                                    <PopupSection close_handle={order_popup.clone()}>
                                        <div class="bg-white rounded-2xl p-6 max-w-xs sm:max-w-sm md:max-w-md lg:mx-w-4xl max-h-96 sm:max-h-[840px] m-4 overflow-y-auto no-scrollbar">
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
                            }
                        </div>
                    }
                }).collect::<Html>()}
            </div>
        </div>
    }
}
