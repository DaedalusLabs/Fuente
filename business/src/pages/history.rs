use crate::contexts::OrderDataStore;

use fuente::mass::HistoryIcon;
use yew::prelude::*;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let order_ctx = use_context::<OrderDataStore>().expect("No order context found");
    let orders = order_ctx.order_history();
    html! {
        <div class="flex flex-col flex-1">
            <h2 class="text-4xl">{"Order History"}</h2>
            {if orders.is_empty() {
                html! {
                    <BlankHistory />
                }
            } else {
                html! {
                    <div class="flex flex-col w-full h-full overflow-y-scroll">
                        {for orders.iter().map(|order| {
                            let order_req = order.get_order_request();
                            let address = order_req.address;
                            let profile = order_req.profile;

                            html! {
                                <div class="flex flex-col w-full p-4 border-b border-neutral-200">
                                    <div class="flex justify-between items-center">
                                        <h4 class="text-lg font-semibold">{order.id()}</h4>
                                        <p class="text-sm text-neutral-400">{profile.nickname()}</p>
                                    </div>
                                    <div class="flex flex-col mt-2">
                                        <p class="text-sm text-neutral-400">
                                            {address.lookup().display_name()}
                                        </p>
                                    </div>
                                </div>
                            }
                        })}
                    </div>
                }
            }}
        </div>
    }
}
#[function_component(BlankHistory)]
pub fn history_page() -> Html {
    html! {
        <>
            <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                <HistoryIcon class="w-32 h-32 stroke-neutral-200" />
                <h4 class="text-xl font-semibold mt-4">{"No history yet"}</h4>
                <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48  text-center text-wrap">
                    {"New sales will appear here!"}
                </p>
            </div>
        </>
    }
}
