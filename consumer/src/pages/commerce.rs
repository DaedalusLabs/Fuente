use crate::contexts::{
    CartAction, CartStore, CommerceDataStore, ConsumerDataStore, LiveOrderStore,
};
use crate::router::ConsumerRoute;

use super::PageHeader;
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{AppLink, CardComponent, ProductCard, SpinnerIcon},
    models::{ProductItem, ProductOrder},
};
use lucide_yew::{ArrowLeft, ShoppingCart};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct CommercePageProps {
    pub commerce_id: String,
}

#[function_component(CommercePage2)]
pub fn history_page(props: &CommercePageProps) -> Html {
    let CommercePageProps { commerce_id } = props;
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    let cart_ctx = Rc::new(use_context::<CartStore>().expect("No cart context found"));
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let live_ctx = use_context::<LiveOrderStore>().expect("LiveOrder context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Consumer context not found");
    let sent_order_request = use_state(|| None::<String>);
    let show_warning = use_state(|| false);
    let menu = commerce_ctx
        .products_lists()
        .iter()
        .find(|p| p.id() == *commerce_id)
        .cloned();
    let add_cart = cart_ctx.clone();
    let sender = relay_ctx.send_note.clone();

    let id = commerce_id.clone();
    let profile = user_ctx.get_profile();
    let address = user_ctx.get_default_address();
    let sent_handle = sent_order_request.clone();
    let live_handle = live_ctx.clone();
    let send_order_request = {
        let cart_ctx = Rc::clone(&cart_ctx);
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

    let commerce_id = commerce_id.clone();
    let onsubmit = {
        let cart_ctx = Rc::clone(&cart_ctx);
        let show_warning = show_warning.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let form = HtmlForm::new(e).expect("Could not capture form");
            let product_str = form.input_value("product").expect("Could not get product");
            let product: ProductItem = product_str.try_into().expect("Could not parse product");
            if !cart_ctx.can_add_from_business(&commerce_id) {
                show_warning.set(true);
                return;
            }
            add_cart.dispatch(CartAction::AddProduct(product, commerce_id.clone()));
        })
    };
    if menu.is_none() {
        return html! {
            <div class="h-full w-full flex flex-col">
                <PageHeader title={"Commerce".to_string()} />
                <div class="flex flex-1 flex-col gap-4">
                    <span class="text-lg font-bold">{"No products found"}</span>
                    <span class="text-neutral-400">{"Add some products to your commerce"}</span>
                </div>
            </div>
        };
    }
    let menu = menu.unwrap().menu().categories();
    if let Some(request) = sent_order_request.as_ref() {
        return html! {
            <div class="h-full w-full flex flex-col">
                <PageHeader title={"Commerce".to_string()} />
                <div class="flex flex-1 flex-col gap-4">
                    <span class="text-lg font-bold">{"Waiting to Confirm your Order"}</span>
                    <span class="text-neutral-400">{"Order ID: "}{request}</span>
                    <SpinnerIcon class="w-8 h-8" />
                </div>
            </div>
        };
    }
    html! {
        <div class="h-full w-full flex flex-col">
        <PageHeader title={"Commerce".to_string()} />
        <div class="flex flex-1 flex-col gap-4">
            {if *show_warning {
                html! {
                    <div class="fixed top-4 right-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded w-80">
                        <span class="block sm:inline">{"You have items from another store in your cart. Please clear your cart first."}</span>
                        <button
                            onclick={let show_warning = show_warning.clone();
                                Callback::from(move |_| show_warning.set(false))}
                            class="absolute top-0 right-0 px-4 py-3"
                        >
                            {"×"}
                        </button>
                    </div>
                }
            } else {
                html! {}
            }}
            <button
                class="bg-fuente-light text-white p-2 rounded-md font-mplus m-4"
                onclick={send_order_request}>
                {"Send Order Request"}
            </button>
            <CartDetails />
            {menu.iter().map(|category| {
               html! {
                   <div class="flex flex-col gap-2 px-4">
                       <h3 class="text-lg font-bold">{category.name().clone()}</h3>
                       <div class="flex flex-col gap-4">
                        {category.products().iter().map(|product| {
                            let onsubmit = onsubmit.clone();
                            let product_str = product.to_string();
                            let name = "product";
                            html! {
                                <form {onsubmit} >
                                    <input class="hidden" {name} value={product_str} />
                                    <button
                                        type="submit">
                                        <ProductCard product={product.clone()} />
                                    </button>
                                </form>
                            }
                        }).collect::<Html>()}
                        </div>
                       </div>
                   }
                }).collect::<Html>()}
            </div>
        </div>
    }
}

#[function_component(CartDetails)]
pub fn cart_details() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let cart = ProductOrder::new(cart_ctx.cart());
    let counted = cart.counted_products();
    if cart.is_empty() {
        return html! {
            <div class="flex flex-col gap-4 p-4">
                <span class="text-lg font-bold">{"Cart is empty"}</span>
                <span class="text-neutral-400">{"Add some products to your cart"}</span>
            </div>
        };
    }
    html! {
        <div class="p-4" >
            <CardComponent>
              <table class="w-full mb-4">
                <thead>
                  <tr class="border-b border-dashed border-gray-300">
                    <th class="text-left py-2">{"Item"}</th>
                    <th class="text-right py-2">{"Qty"}</th>
                    <th class="text-right py-2">{"Price"}</th>
                  </tr>
                </thead>
                <tbody>
                    {counted.iter().map(|(product, count)| {
                        let subtotal = product.price().parse::<f64>().unwrap() * *count as f64;
                        html! {
                            <tr key={product.id()} class="border-b border-dotted border-gray-200">
                                <td class="py-2">{product.name()}</td>
                                <td class="text-right py-2">{count}</td>
                                <td class="text-right py-2">{format!("{:.2}",subtotal)}</td>
                            </tr>
                        }
                    }).collect::<Html>()}
                </tbody>
              </table>
              <div class="flex justify-between font-bold border-t-2 border-black pt-2">
                <span>{"Total"}</span>
                <span>{format!("{:.2}", cart.total())}</span>
              </div>
            </CardComponent>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct CartItemProp {
    pub product: ProductItem,
    pub count: u32,
}

#[function_component(CartItem)]
pub fn cart_item(props: &CartItemProp) -> Html {
    let CartItemProp { product, count } = props;
    let price = product.price().parse::<f64>().unwrap();
    let total = price * *count as f64;
    html! {
        <CardComponent>
            <div class="flex flex-row gap-4 relative p-4">
                <div class="w-16 h-16 bg-neutral-200 rounded-2xl"></div>
                <div class="flex flex-col">
                    <span class="font-bold text-lg mb-1">{product.name()}</span>
                    <span class="text-neutral-400">{product.price()}</span>
                </div>
                <div class="flex flex-col gap-4">
                    <span class="text-neutral-400">{props.count}</span>
                    <span class="text-neutral-400">{total}</span>
                </div>
            </div>
        </CardComponent>
    }
}
#[function_component(CommercePage)]
pub fn commerce_page_template(props: &CommercePageProps) -> Html {
    let CommercePageProps { commerce_id } = props;
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();

    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    let commerce_profile = commerce_ctx
        .find_commerce_by_id(commerce_id)
        .expect("No commerce found");
    let products = commerce_ctx
        .find_product_list_by_id(commerce_id)
        .expect("No products found");
    let products = products.menu();
    let commerce_profile = commerce_profile.profile();

    let product_handle = use_state(|| None::<ProductItem>);
    let show_warning = use_state(|| false);
    match product_handle.as_ref() {
        Some(product) => html! {
            <ProductPage product={product.clone()} commerce_id={commerce_id.clone()} product_handle={product_handle.clone()} />
        },
        None => html! {
            <>
            <section class="grid grid-cols-[1fr_2fr] gap-5 container mx-auto mt-12 place-items-center">
                <h1 class="text-6xl uppercase text-fuente tracking-tighter font-bold text-center">{&commerce_profile.name}</h1>

                <div class="xl:relative flex items-center justify-center bg-fuente rounded-2xl h-60 max-h-60 pr-5 w-full overflow-hidden">
                    <img src={commerce_profile.banner_url.clone()} alt={commerce_profile.name.clone()}
                        class="xl:absolute xl:-top-5 xl:left-0 xl:-translate-x-10 xl:-translate-y-10 block object-contain lg:max-w-56 xl:max-w-full" />
                </div>
            </section>

            <main class="mt-4 container mx-auto">
                        <div class="grid grid-cols-[1fr_3fr] gap-10 mt-10">
                            <aside class="flex flex-col gap-3 bg-gray-100 p-10 rounded-2xl h-fit">
                                <h3 class="font-semibold text-fuente text-xl">{&translations["detail_store_filter_heading"]}</h3>
                                <div>
                                    <p class="text-fuente p-2 font-light text-lg border-b border-b-fuente">{&translations["detail_store_filter_price"]}</p>
                                    <p class="text-fuente p-2 font-light text-lg">{&translations["detail_store_filter_brand"]}</p>
                                </div>
                            </aside>

                            <div class="flex-1 grid lg:grid-cols-2 xl:grid-cols-3 gap-5 place-items-center">
                                {products.categories().iter().map(|category| {
                                    category.products().iter().map(|product| {
                                        let onclick = {
                                            let product_handle = product_handle.clone();
                                            let product = product.clone();
                                            Callback::from(move |_e: MouseEvent| {
                                                product_handle.set(Some(product.clone()));
                                            })
                                        };
                                        html! {
                                            <button {onclick}>
                                                <ProductItemCard product={product.clone()} commerce_id={commerce_id.clone()} product_handle={product_handle.clone()} />
                                            </button>
                                        }
                                    }).collect::<Html>()
                                }).collect::<Html>()}
                            </div>
                        </div>
            </main>
            </>
        },
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ProductItemProps {
    pub product: ProductItem,
    pub commerce_id: String,
    pub product_handle: UseStateHandle<Option<ProductItem>>,
}

#[function_component(ProductItemCard)]
pub fn product_item_card(props: &ProductItemProps) -> Html {
    let ProductItemProps {
        product,
        commerce_id,
        product_handle,
    } = props;
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let show_warning = use_state(|| false);
    let onclick = {
        let commerce_id = commerce_id.clone();
        let cart_ctx = cart_ctx.clone();
        let show_warning = show_warning.clone();
        let product = product.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if !cart_ctx.can_add_from_business(&commerce_id) {
                show_warning.set(true);
                return;
            }
            cart_ctx.dispatch(CartAction::AddProduct(
                product.clone(),
                commerce_id.to_string(),
            ));
        })
    };
    let image_onclick = {
        let product_handle = product_handle.clone();
        let product = product.clone();
        Callback::from(move |_e: MouseEvent| {
            product_handle.set(Some(product.clone()));
        })
    };
    html! {
        <div class="border border-fuente rounded-2xl p-2 max-w-72 max-h-96 overflow-hidden">
            <div class="relative">
                <img onclick={image_onclick} src={product.image_url()} alt="Favorites Image"
                class="object-contain w-full max-h-52 mx-auto bg-gray-100 rounded-2xl" />
                // add favorite items?
            </div>
            <h2 class="font-bold text-lg text-gray-500 text-center mt-3">{product.name()}</h2>
            <p class="text-sm text-gray-400 text-center line-clamp-2">{product.details()}</p>
            <div class="flex justify-between items-center mt-3 px-5 gap-5">
                <p class="text-xl font-bold text-fuente">{format!("SRD {}", product.price())}</p>
                <button {onclick} class="bg-fuente-orange text-white py-2 px-7 rounded-full z-[500]">
                    <ShoppingCart class="w-8 h-8" />
                </button>
            </div>
        </div>
    }
}

#[function_component(ProductPage)]
pub fn product_page_template(props: &ProductItemProps) -> Html {
    let ProductItemProps {
        product,
        commerce_id,
        product_handle,
    } = props;
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let back_to_store = {
        let product_handle = product_handle.clone();
        Callback::from(move |_e: MouseEvent| {
            product_handle.set(None);
        })
    };
    let add_cart = {
        let cart_ctx = cart_ctx.clone();
        let commerce_id = commerce_id.clone();
        let product = product.clone();
        Callback::from(move |_e: MouseEvent| {
            cart_ctx.dispatch(CartAction::AddProduct(product.clone(), commerce_id.clone()));
        })
    };
    html! {
        <main class="mt-20 container mx-auto">
            <button onclick={back_to_store}
                class="flex gap-3">
                <ArrowLeft class="w-8 h-8 text-fuente" />
                <p class="text-fuente text-lg font-semibold">{"Back to store"}</p>
            </button>

            <div class="grid grid-cols-[3fr_1fr] items-center mt-10">
                <div class="grid grid-cols-[3fr_1fr] place-items-center">
                    <img src={product.image_url()} alt={product.name()} class="bg-gray-100 rounded-2xl w-5/6 object-contain h-full" />

                    <div class="flex flex-col gap-4 justify-between">
                        <img src={product.thumbnail_url()} alt="Sneaker Product" class="w-40 bg-gray-100 rounded-2xl block object-contain flex-1" />
                    </div>
                </div>

                <div>
                    <h2 class="text-gray-500 text-2xl font-bold">{product.name()}</h2>
                    <p class="font-light text-gray-500 text-xl mt-3">{product.description()}</p>
                    <p class="font-bold text-gray-500 uppercase text-2xl">{format!("SKU {}", product.sku())}</p>
                    // <div class="flex items-center gap-1 mt-10">
                    //     <svg baseProfile="tiny" version="1.2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 ">
                    //         <path d="m9.362 9.158-5.268.584c-.19.023-.358.15-.421.343s0 .394.14.521c1.566 1.429 3.919 3.569 3.919 3.569-.002 0-.646 3.113-1.074 5.19a.496.496 0 0 0 .734.534c1.844-1.048 4.606-2.624 4.606-2.624l4.604 2.625c.168.092.378.09.541-.029a.5.5 0 0 0 .195-.505l-1.071-5.191 3.919-3.566a.499.499 0 0 0-.28-.865c-2.108-.236-5.269-.586-5.269-.586l-2.183-4.83a.499.499 0 0 0-.909 0l-2.183 4.83z" fill="#4167e8" class="fill-000000"></path>
                    //     </svg>
                    //     <p class="text-gray-500 text-2xl">{"5.0 (30 reviews)"}</p>
                    // </div>

                    <div class="flex flex-col mt-10 px-5 gap-5">
                        <p class="text-5xl font-bold text-fuente">{format!("SRD {}", product.price())}</p>
                        <button onclick={add_cart} 
                            class="bg-fuente-orange text-white py-4 px-10 rounded-full flex items-center justify-center gap-2 w-5/6">
                            <ShoppingCart class="w-8 h-8" />
                            <p class="font-semibold text-xl text-center">{"Shop Now"}</p>
                        </button>
                    </div>
                </div>
            </div>

            <section class="bg-gray-100 px-10 py-16 rounded-2xl mt-5 space-y-7">
                <p class="text-gray-500 leading-6">{product.description()}</p>
            </section>
        </main>
    }
}
