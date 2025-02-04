use crate::contexts::{
    CartAction, CartStore, CommerceDataStore, ConsumerDataStore, LiveOrderStore, LoginStateAction, LoginStateStore,
};
use crate::pages::OrderInvoiceComponent;
use crate::router::ConsumerRoute;
use fuente::contexts::{AdminConfigsStore, LanguageConfigsStore};
use fuente::mass::{ThreeBlockSpinner, AppLink};
use fuente::models::{OrderPaymentStatus, ProductItem, ProductOrder};
use lucide_yew::{ArrowRight, Trash2};
use nostr_minions::key_manager::NostrIdStore;
use nostr_minions::relay_pool::NostrProps;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[function_component(CartPage)]
pub fn cart_page() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("No language context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("No key context not found");
    let translations = language_ctx.translations();
    let login_state = use_context::<LoginStateStore>().expect("LoginStateStore not found");

    let cart_items = cart_ctx.cart();
    if cart_items.is_empty() {
        return html! {
            <EmptyCart />
        };
    }

    if key_ctx.get_nostr_key().is_none() {
        // Show login modal and return placeholder
        login_state.dispatch(LoginStateAction::Show);
        return html! {
            <div class="h-screen flex items-center justify-center">
                <p>{"Please log in to access your cart"}</p>
            </div>
        };
    }

    html! {
        <main class="flex flex-col h-screen overflow-hidden w-full mx-auto">
            <div class="flex flex-col lg:flex-row justify-between items-center px-4 lg:px-10 gap-4">
                <h1 class="text-2xl lg:text-6xl text-nowrap uppercase text-fuente tracking-tighter font-bold text-center">
                    {&translations["cart_heading"]}
                </h1>
            </div>
            <div class="flex-grow flex flex-col lg:flex-row overflow-hidden">
               <CartTemplate order={ProductOrder::new(cart_items)} />
               <CartPreTotal />
            </div>
        </main>
    }
}

#[function_component(EmptyCart)]
pub fn empty_cart() -> Html {
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("No language context not found");
    let translations = language_ctx.translations();
    html! {
        <main class="flex flex-col h-screen overflow-hidden w-full mx-auto">
            <div class="flex flex-col lg:flex-row justify-between items-center px-4 lg:px-10 gap-4">
                <h1 class="text-2xl lg:text-6xl text-nowrap uppercase text-fuente tracking-tighter font-bold text-center">
                    {&translations["checkout_product_empty_table_heading"]}
                </h1>
            </div>
            <div class="container bg-fuente rounded-2xl p-5 flex flex-col mx-auto h-fit w-fit">
                <div class="flex justify-between items-center lg:mb-4">
                    <h2 class="text-white text-4xl font-semibold tracking-tighter">{&translations["home_stores"]}</h2>
                    <AppLink<ConsumerRoute>
                        class=""
                        selected_class=""
                        route={ConsumerRoute::BrowseStores}>
                        <ArrowRight class="w-12 h-12 text-white rounded-full border-4 border-white" />
                    </AppLink<ConsumerRoute>>
                </div>

                <img src="/public/assets/img/store.png" alt="Store Image" class="object-contain w-64 mx-auto " />
            </div>
        </main>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct CartTemplateProps {
    pub order: ProductOrder,
}

#[function_component(CartTemplate)]
pub fn checkout_cart_template(props: &CartTemplateProps) -> Html {
    let CartTemplateProps { order } = props;
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("No language context not found");
    let translations = language_ctx.translations();
    let counted_products = order.counted_products();
    if counted_products.is_empty() {
        return html! {
            <div class="border border-fuente mt-7 px-16 rounded-3xl flex items-center justify-center">
                <h2 class="text-lg text-fuente font-bold p-5">{&translations["checkout_product_empty_table_heading"]}</h2>
            </div>
        };
    }
    html! {
        <div class="flex-1 overflow-hidden px-4 py-2 lg:py-4 w-full">
            <div class="h-full overflow-auto border border-fuente rounded-xl no-scrollbar relative p-2">
                <h2 class=" bg-white flex text-2xl text-fuente font-bold pt-5 sticky top-0">{&translations["cart_text"]}</h2>

                <div class="hidden lg:flex justify-between items-center lg:mt-10 xl:mt-5">
                    <h3></h3>
                    <h3 class="text-fuente lg:pl-16 xl:pl-40">{&translations["cart_table_heading_details"]}</h3>
                    <h3 class="text-fuente lg:pl-0 xl:pl-5">{&translations["cart_table_heading_quantity"]}</h3>
                    <h3 class="text-fuente lg:pr-10 xl:pr-32">{&translations["cart_table_heading_price"]}</h3>
                    <h3></h3>
                </div>
                {order.counted_products().iter().map(|(item, count)| {

                    html! {
                        <CartItemDetails
                            item={item.clone()}
                            count={*count} />
                    }
                }).collect::<Html>()}
            </div>
        </div>
    }
}
#[function_component(CartPreTotal)]
pub fn cart_pre_total() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No key context not found");
    let relay_ctx = use_context::<NostrProps>().expect("No relay context not found");
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("No language context not found");
    let translations = language_ctx.translations();
    let order = ProductOrder::new(cart_ctx.cart());
    let id = cart_ctx.business_id().expect("No business id");
    let profile = user_ctx.get_profile();
    let address = user_ctx.get_default_address();
    let navigator = use_navigator().expect("No navigator found");

    let send_order_request = {
        let cart_ctx = cart_ctx.clone();
        let sender = relay_ctx.send_note.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let keys = key_ctx.get_nostr_key();
            let note = cart_ctx.sign_request(
                &keys.unwrap(),
                id.clone(),
                profile.clone().unwrap(),
                address.clone().unwrap(),
            );
            // sent_handle.set(Some(note.id.as_ref().unwrap().to_string()));
            sender.emit(note.1);
            cart_ctx.dispatch(CartAction::SentOrder(note.0));
            navigator.push(&ConsumerRoute::Checkout);
        })
    };
    html! {
        <div class="flex flex-col gap-4 mx-auto">
            <div class="bg-gray-100 p-5 lg:px-12 m-5 rounded-2xl flex justify-end items-center">
                <p class="text-center text-fuente text-lg flex items-center gap-10">
                    {&translations["cart_pre_total"]}
                    <span class="font-bold text-2xl md:text-3xl">{format!("SRD {:.2}", order.total())}</span>
                </p>
            </div>

            <div class="lg:flex lg:justify-center my-3 px-5 lg:px-12">
                <button onclick={send_order_request}
                    class="bg-fuente-buttons text-lg w-full lg:w-fit text-nowrap py-4 px-10 rounded-full font-bold text-fuente-forms">
                    {&translations["cart_checkout"]}
                </button>
            </div>
        </div>
    }
}

#[function_component(CheckoutPage)]
pub fn cart_page() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("No language context not found");
    let translations = language_ctx.translations();
    let order_id = cart_ctx.last_sent_order().unwrap_or_default();

    let cart_items = cart_ctx.cart();
    if cart_items.is_empty() {
        return html! {};
    }

    html! {
        <main class="flex flex-col h-screen overflow-hidden w-full">
            <div class="flex flex-col lg:flex-row justify-between items-center px-4 lg:px-10 gap-4">
                <h1 class="text-2xl lg:text-6xl text-nowrap uppercase text-fuente tracking-tighter font-bold text-center">
                    {&translations["checkout_title"]}
                </h1>
            </div>
            <div class="flex-grow flex flex-col lg:flex-row overflow-hidden">
                <div class="grid xl:grid-cols-[3fr_1fr] mt-10 gap-5 overflow-y-auto">
                    <div>
                        <CheckoutClientInfo />
                        <CartTemplate order={ProductOrder::new(cart_items)} />
                    </div>

                    <div>
                        <CheckoutInvoice order_id={order_id} />
                        <CheckoutOrderSummary />
                    </div>
                </div>
            </div>
        </main>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct CartItemDetailsProps {
    pub item: ProductItem,
    pub count: u32,
}

#[function_component(CartItemDetails)]
pub fn cart_item_details(props: &CartItemDetailsProps) -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let CartItemDetailsProps { item, count } = props;
    let remove_one_item = {
        let cart_ctx = cart_ctx.clone();
        let item_clone = item.clone();
        Callback::from(move |_: MouseEvent| {
            cart_ctx.dispatch(CartAction::RemoveProduct(item_clone.clone()));
        })
    };
    let add_one_item = {
        let cart_ctx = cart_ctx.clone();
        let item_clone = item.clone();
        Callback::from(move |_: MouseEvent| {
            cart_ctx.dispatch(CartAction::AddOne(item_clone.clone()));
        })
    };
    let clear_product = {
        let cart_ctx = cart_ctx.clone();
        let item_clone = item.clone();
        Callback::from(move |_: MouseEvent| {
            cart_ctx.dispatch(CartAction::ClearProduct(item_clone.clone()));
        })
    };
    let price = item.price().parse::<f64>().unwrap() * *count as f64;
    html! {
        <div class="flex justify-between  items-center gap-5 md:gap-20 mt-10 py-10 border-t border-t-fuente">
            <img
                src={item.thumbnail_url()}
                alt="Product Image"
                class="w-20 sm:w-28 lg:w-32 object-contain bg-gray-100 rounded-xl block"
            />
            <div class="flex flex-col">
                <p class="text-gray-500 font-bold">{item.name()}</p>
                <p class="text-gray-500 font-light w-20 sm:w-28 lg:w-32 line-clamp-3">{item.details()}</p>
                <p class="text-gray-500 font-bold uppercase">{format!("SKU: {}", item.sku())}</p>
                <button onclick={add_one_item.clone()}
                    class="lg:hidden border-2 border-fuente px-5 py-2 rounded-xl w-fit mt-1">{count}</button>
            </div>

            <div class="hidden lg:flex items-center justify-between border border-fuente rounded-xl">
                <button onclick={remove_one_item}
                    class="text-gray-500 w-full px-5 py-3">{"-"}</button>
                <button class="text-gray-500 w-full px-5 py-3">{count}</button>
                <button onclick={add_one_item}
                    class="text-gray-500 w-full px-5 py-3">{"+"}</button>
            </div>

            <p class="text-2xl md:text-4xl text-fuente font-bold">{format!("SRD {:.2}", price)}</p>

            <button onclick={clear_product} >
                <Trash2 class="w-8 h-8 text-red-500" />
            </button>
        </div>
    }
}

#[function_component(CheckoutClientInfo)]
pub fn checkout_summary() -> Html {
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language ctx found");
    let translations = language_ctx.translations();
    let user_profile = user_ctx.get_profile();
    let user_address = user_ctx.get_default_address();
    if let (Some(profile), Some(address)) = (user_profile, user_address) {
        html! {
            <div class="border border-fuente rounded-2xl py-5 px-10 mx-2 lg:mx-4">
                <h2 class="text-fuente text-3xl font-semibold">{&translations["checkout_client_information"]}</h2>
                <div class="flex items-center justify-between mt-5 flex-wrap gap-5">
                    <div>
                        <h3 class="text-gray-400 font-bold text-lg">{&translations["checkout_client_information_heading_name"]}</h3>
                        <p class="font-light text-gray-400 text-md">{&profile.nickname}</p>
                    </div>
                    <div>
                        <h3 class="text-gray-400 font-bold text-lg">{&translations["checkout_client_information_heading_email"]}</h3>
                        <p class="font-light text-gray-400 text-md">{&profile.email}</p>
                    </div>
                    <div>
                        <h3 class="text-gray-400 font-bold text-lg">{&translations["checkout_client_information_heading_phone"]}</h3>
                        <p class="font-light text-gray-400 text-md">{&profile.telephone}</p>
                    </div>
                    <div>
                        <h3 class="text-gray-400 font-bold text-lg">{&translations["checkout_client_information_heading_location"]}</h3>
                        <p class="font-light text-gray-400 text-md">{address.lookup().display_name()}
                        </p>
                    </div>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="border border-fuente rounded-2xl py-5 px-10">
                <h2 class="text-fuente text-3xl font-semibold">{"Info not set"}</h2>
            </div>
        }
    }
}
#[function_component(CheckoutOrderSummary)]
pub fn checkout_summary() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce ctx");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language ctx found");
    let translations = language_ctx.translations();
    let order = ProductOrder::new(cart_ctx.cart());
    let business = commerce_ctx
        .find_commerce_by_id(cart_ctx.business_id().expect("No business id").as_str())
        .expect("No business found");
    let business = business.profile();
    html! {
        <div class="bg-zinc-100 py-7 px-10 rounded-2xl mt-7 mx-2 lg:mx-4">
            <h2 class="text-fuente text-3xl font-bold mt-7">{&translations["checkout_summary_heading"]}</h2>

            <div class="border-y border-y-fuente mt-7 space-y-5 py-5">
                <div class="space-y-2">
                    <h3 class="text-gray-500 font-bold text-lg">{&translations["checkout_summary_price_details_pre_total"]}</h3>
                    <p class="text-gray-400 text-lg font-light">{format!("SRD {:.2}", order.total())}</p>
                </div>
                <div class="space-y-2">
                    <h3 class="text-gray-500 font-bold text-lg">{&translations["checkout_summary_price_details_fee"]}</h3>
                    <p class="text-gray-400 text-lg font-light">{"Free"}</p>
                </div>
                // <div class="space-y-2">
                //     <h3 class="text-gray-500 font-bold text-lg">{"Taxes"}</h3>
                //     <p class="text-gray-400 text-lg font-light">{"$13.00"}</p>
                // </div>
            </div>

            <div class="mt-7 space-y-5">
                <div class="space-y-2">
                    <h3 class="text-gray-500 font-bold text-lg">{&translations["checkout_summary_total_products_heading"]}</h3>
                    <p class="text-gray-400 text-lg font-light">{format!("{} products", order.products().len())}</p>
                </div>

                <div class="space-y-2">
                    <h3 class="text-gray-500 font-bold text-lg">{&translations["checkout_summary_pickup_location_heading"]}</h3>
                    <p class="text-gray-400 text-lg font-light">{&business.name}</p>
                    <p class="text-gray-400 text-lg font-light line-clamp-3">{&business.lookup.display_name()}</p>
                </div>

                // <div class="space-y-2">
                //     <h3 class="text-gray-500 font-bold text-lg">{"Pickup date"}</h3>
                //     <p class="text-gray-400 text-lg font-light">{"12/12/24"}</p>
                // </div>
            </div>

        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct OrderInvoiceProps {
    pub order_id: String,
}

#[function_component(CheckoutInvoice)]
pub fn checkout_summary(props: &OrderInvoiceProps) -> Html {
    let admin_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let cart_ctx = use_context::<CartStore>().expect("CartStore not found");
    let navigator = use_navigator().unwrap();
    let exchange_rate = admin_ctx.get_exchange_rate();
    let order_id = props.order_id.clone();

    {
        let navigator = navigator.clone();
        use_effect_with(order_ctx.live_orders.clone(), move |order| {
            if let Some((_, order_state)) = order.last() {
                match order_state.payment_status {
                    OrderPaymentStatus::PaymentReceived => {
                        // Changed from PaymentPending
                        if let Some(order_id) = order_state.order.id.clone() {
                            cart_ctx.dispatch(CartAction::ClearCart);
                            navigator.push(&ConsumerRoute::Order { order_id });
                        }
                    }
                    _ => {}
                }
            }
            || {}
        });
    }

    if let Some(order) = order_ctx.live_orders.iter().find(|o| o.1.order_id() == order_id) {
        html! {
            <OrderInvoiceComponent
                invoice={order.1.consumer_invoice.as_ref().cloned().unwrap()}
                {exchange_rate}
            />
        }
    } else {
        html! {
            <div class="bg-zinc-100 p-4 rounded-2xl flex flex-col gap-3 items-center justify-center w-full">
                <ThreeBlockSpinner class="w-8 h-8 text-fuente" />
            </div>
        }
    }
}
