use crate::contexts::{
    CartAction, CartStore, CommerceDataStore, ConsumerDataStore, LiveOrderStore,
};
use crate::pages::OrderInvoiceComponent;
use fuente::contexts::{AdminConfigsStore, LanguageConfigsStore};
use fuente::models::ProductOrder;
use lucide_yew::Trash;
use yew::prelude::*;

#[function_component(CartPage)]
pub fn cart_page() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");

    let cart_items = cart_ctx.cart();
    if cart_items.is_empty() {
        return html! {};
    }

    html! {
    <main class="container mx-auto mt-10">
        <h1 class="text-fuente text-6xl tracking-tighter font-bold ">{"Checkout"}</h1>
        <div class="grid xl:grid-cols-[3fr_1fr] mt-10 gap-5">
            <div>
                <CheckoutClientInfo />
                <CheckoutCartTemplate order={ProductOrder::new(cart_items)} />
            </div>

            <div>
                <CheckoutInvoice />
                <CheckoutOrderSummary />
            </div>
        </div>
    </main>
    }
}
#[derive(Properties, Clone, PartialEq)]
pub struct CheckoutCartTemplateProps {
    #[prop_or_default]
    pub children: Children,
    pub order: ProductOrder,
}

#[function_component(CheckoutCartTemplate)]
pub fn checkout_cart_template(props: &CheckoutCartTemplateProps) -> Html {
    let CheckoutCartTemplateProps { children: _, order } = props;
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
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
        <div class="border border-fuente mt-7 px-16 rounded-3xl">
            <h2 class="text-2xl text-fuente font-bold pt-5">{&translations["packages_track_table_heading_details"]}</h2>

            <table class="table-auto w-full border-collapse">
                <thead>
                    <tr>
                        <th></th>
                        <th class=" py-3 text-left text-md leading-4 font-semibold text-fuente text-lg">
                            {&translations["checkout_product_details_table_heading"]}</th>
                        <th class=" py-3 text-center text-md leading-4 font-semibold text-fuente text-lg">
                            {&translations["checkout_product_quantity_table_heading"]}</th>
                        <th class=" py-3 text-center text-md leading-4 font-semibold text-fuente text-lg">
                            {&translations["checkout_product_price_table_heading"]}</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {order.counted_products().iter().map(|(item, count)| {
                        let remove_one_item = {
                            let cart_ctx = cart_ctx.clone();
                            let item_clone = item.clone();
                            Callback::from(move |_| {
                            cart_ctx.dispatch(CartAction::RemoveProduct(item_clone.clone()));
                        })};
                        let add_one_item = {
                            let cart_ctx = cart_ctx.clone();
                            let item_clone = item.clone();
                            Callback::from(move |_| {
                            cart_ctx.dispatch(CartAction::AddOne(item_clone.clone()));
                        })};
                        let clear_product = {
                            let cart_ctx = cart_ctx.clone();
                            let item_clone = item.clone();
                            Callback::from(move |_| {
                            cart_ctx.dispatch(CartAction::ClearProduct(item_clone.clone()));
                        })
                        };
                        let price = item.price().parse::<f64>().unwrap() * *count as f64;

                        html! {
                            <tr>
                                <td class="py-8 pt-4 whitespace-nowrap">
                                    <img src={item.thumbnail_url()} alt="Product Image" class="w-32 min-w-32 bg-gray-200 rounded-2xl" />
                                </td>
                                <td class="px-6 py-8 whitespace-nowrap overflow-hidden truncate">
                                    <p class="font-bold text-gray-500 mt-8">{item.name()}</p>
                                    <p class="font-thin text-gray-500 mt-3 max-w-32 truncate">{item.details()}</p>
                                    <p class="font-bold text-gray-500">{format!("SKU: {}", item.sku())}</p>
                                </td>
                                <td class="px-6 py-8">
                                    <div
                                        class="border border-fuente flex justify-center gap-8 rounded-xl py-3 px-2 mt-20">
                                        <button onclick={remove_one_item} class="text-gray-600">{"-"}</button>
                                        <span class="text-gray-600">{count}</span>
                                        <button onclick={add_one_item} class="text-gray-600">{"+"}</button>
                                    </div>
                                </td>
                                <td class="px-6 py-8 whitespace-nowrap text-right text-4xl text-fuente font-semibold">
                                    <p class="mt-20">{format!("{:.2}", price)}</p>
                                </td>
                                <td class="whitespace-nowrap text-center py-8 px-6">
                                    <button onclick={clear_product} class="w-10 h-10 mt-24">
                                        <Trash class="w-10 h-10 text-fuente" />
                                    </button>
                                </td>
                            </tr>
                        }
                    }).collect::<Html>()}
                </tbody>
            </table>
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
            <div class="border border-fuente rounded-2xl py-5 px-10">
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
        <div class="bg-zinc-100 py-7 px-10 rounded-2xl mt-7">
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
#[function_component(CheckoutInvoice)]
pub fn checkout_summary() -> Html {
    let admin_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let exchange_rate = admin_ctx.get_exchange_rate();
    if let Some(order) = order_ctx.order.as_ref() {
        html! {
            <OrderInvoiceComponent invoice={order.1.consumer_invoice.as_ref().cloned().unwrap()} {exchange_rate} />
        }
    } else {
        html! {}
    }
}
