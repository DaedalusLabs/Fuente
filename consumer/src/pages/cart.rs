use crate::contexts::{CartAction, CartStore, ConsumerDataStore};
use crate::pages::PageHeader;
use crate::router::ConsumerRoute;
use fuente::mass::{AppLink, CardComponent, ShoppingCartIcon, SimpleFormButton, SpinnerIcon};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;
use fuente::models::ProductOrder;
use fuente::mass::OrderRequestDetailsComponent;

#[function_component(CartPage)]
pub fn cart_page() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let cart_items = cart_ctx.cart();
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Consumer context not found");
    let sent_order_request = use_state(|| None::<String>);
    
    if cart_items.is_empty() {
        return html! {
            <div class="h-full w-full flex flex-col justify-between items-center">
                <PageHeader title={"Cart".to_string()} />
                <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                    <ShoppingCartIcon class="w-32 h-32 stroke-neutral-200" />
                    <h4 class="text-xl font-semibold mt-4">{"Nothing in your cart"}</h4>
                    <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48 text-center text-wrap">
                        {"Hit the button below to create a new order!"}
                    </p>
                </div>
                <AppLink<ConsumerRoute> 
                    route={ConsumerRoute::Home}
                    class="mb-8"
                    selected_class=""
                >
                    <SimpleFormButton>
                        {"Browse Stores"}
                    </SimpleFormButton>
                </AppLink<ConsumerRoute>>
            </div>
        };
    }

    if let Some(request) = sent_order_request.as_ref() {
        return html! {
            <div class="h-full w-full flex flex-col">
                <PageHeader title={"Cart".to_string()} />
                <div class="flex flex-1 flex-col gap-4 items-center justify-center">
                    <h2 class="text-2xl font-bold">{"Order Received!"}</h2>
                    <OrderRequestDetailsComponent 
                        order={ProductOrder::new(cart_ctx.cart())} 
                    />
                    <p>{"Order ID: "}{request}</p>
                    <p>{"Waiting for confirmation..."}</p>
                    <SpinnerIcon class="w-8 h-8 text-fuente" />
                </div>
            </div>
        };
    }

    let total: f64 = cart_items.iter()
        .map(|item| item.price().parse::<f64>().unwrap())
        .sum();

    let clear_cart = {
        let cart_ctx = cart_ctx.clone();
        Callback::from(move |_| {
            cart_ctx.dispatch(CartAction::ClearCart);
        })
    };

    let checkout = {
        let sender = relay_ctx.send_note.clone();
        let cart_ctx = cart_ctx.clone();
        let id = cart_items[0].id().clone();
        let profile = user_ctx.get_profile();
        let address = user_ctx.get_default_address();
        let sent_handle = sent_order_request.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let keys = key_ctx.get_nostr_key();
            let note = cart_ctx.sign_request(
                &keys.unwrap(),
                id.clone(),
                profile.clone().unwrap(),
                address.clone().unwrap(),
            );
            sent_handle.set(Some(note.id.as_ref().unwrap().to_string()));
            sender.emit(note);
        })
    };

    html! {
        <div class="h-full w-full flex flex-col">
            <PageHeader title={"Cart".to_string()} />
            <div class="flex-1 overflow-auto p-4">
                <div class="max-w-2xl mx-auto space-y-4">
                    {cart_items.iter().map(|item| {
                        let cart_ctx = cart_ctx.clone();
                        let item_clone = item.clone();
                        let remove_item = Callback::from(move |_| {
                            cart_ctx.dispatch(CartAction::RemoveProduct(item_clone.clone()));
                        });

                        html! {
                            <CardComponent>
                                <div class="flex justify-between items-center p-4">
                                    <div>
                                        <h3 class="font-semibold">{item.name()}</h3>
                                        <p class="text-sm text-gray-600">{item.description()}</p>
                                    </div>
                                    <div class="flex items-center gap-4">
                                        <p class="font-medium">{format!("SRD {}", item.price())}</p>
                                        <button 
                                            onclick={remove_item}
                                            class="text-red-500 hover:text-red-700"
                                        >
                                            {"Remove"}
                                        </button>
                                    </div>
                                </div>
                            </CardComponent>
                        }
                    }).collect::<Html>()}
                </div>
            </div>
            
            <div class="border-t bg-white p-4">
                <div class="max-w-2xl mx-auto">
                    <div class="flex justify-between items-center mb-4">
                        <span class="font-semibold">{"Total"}</span>
                        <span class="font-semibold">{format!("SRD {:.2}", total)}</span>
                    </div>
                    <div class="flex gap-4">
                        <button 
                            onclick={clear_cart}
                            class="flex-1 py-2 px-4 border border-red-500 text-red-500 rounded-lg hover:bg-red-50"
                        >
                            {"Clear Cart"}
                        </button>
                        <button 
                            onclick={checkout}
                            class="flex-1 py-2 px-4 bg-fuente text-white rounded-lg hover:bg-fuente-dark"
                        >
                            {"Checkout"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}