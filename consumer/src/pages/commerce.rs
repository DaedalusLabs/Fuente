use crate::contexts::{CartAction, CartStore, CommerceDataStore};

use fuente::{contexts::LanguageConfigsStore, models::ProductItem};
use lucide_yew::{ArrowLeft, ShoppingCart};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct CommercePageProps {
    pub commerce_id: String,
}

#[derive(Clone, PartialEq, Copy)]
pub enum ProductFilter {
    Price,
    Brand,
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
    let product_filter = use_state(|| Option::<ProductFilter>::None);
    let onclick_brand_filter = {
        let product_filter = product_filter.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            match product_filter.as_ref() {
                Some(ProductFilter::Price) => product_filter.set(Some(ProductFilter::Brand)),
                Some(ProductFilter::Brand) => product_filter.set(None),
                None => product_filter.set(Some(ProductFilter::Brand)),
            }
        })
    };
    let onclick_price_filter = {
        let product_filter = product_filter.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            match product_filter.as_ref() {
                Some(ProductFilter::Brand) => product_filter.set(Some(ProductFilter::Price)),
                Some(ProductFilter::Price) => product_filter.set(None),
                None => product_filter.set(Some(ProductFilter::Price)),
            }
        })
    };
    let mut all_products = products
        .categories()
        .iter()
        .flat_map(|category| {
            category
                .products()
                .iter()
                .map(|product| product.clone())
                .collect::<Vec<ProductItem>>()
        })
        .collect::<Vec<ProductItem>>();
    match product_filter.as_ref() {
        Some(ProductFilter::Price) => {
            all_products.sort_by(|a, b| a.price().partial_cmp(&b.price()).unwrap());
        }
        Some(ProductFilter::Brand) => {
            all_products.sort_by(|a, b| a.name().partial_cmp(&b.name()).unwrap());
        }
        None => {}
    }
    match product_handle.as_ref() {
        Some(product) => html! {
            <ProductPage product={product.clone()} commerce_id={commerce_id.clone()} product_handle={product_handle.clone()} />
        },
        None => html! {
            <>
            <section class="grid lg:grid-cols-[1fr_2fr] gap-5 container mx-auto lg:mt-20 place-items-center">
                <h1 class="text-4xl lg:text-9xl uppercase text-fuente tracking-tighter font-bold text-center">
                    {&commerce_profile.name}
                </h1>

                <div class="xl:relative flex items-center justify-center lg:justify-start gap-10 md:gap-0 bg-fuente rounded-2xl h-32 sm:h-44 lg:h-60 pr-5 w-full">
                    <img src={commerce_profile.banner_url.clone()} alt={commerce_profile.name.clone()}
                    class="xl:absolute xl:-top-5 xl:left-0 translate-x-10 lg:translate-x-0 2xl:translate-x-28 xl:-translate-y-10 object-contain w-40 min:[470px]:w-[300px] sm:w-[300px] xl:w-[350px]" />
                </div>
            </section>

            <main class="mt-4 container mx-auto">
                <div class="grid md:grid-cols-[1fr_3fr] gap-10 mt-10 place-content-center justify-items-center md:justify-items-start md:place-items-start">
                    <aside 
                        class="flex flex-row lg:flex-col gap-3 bg-gray-100 p-2 sm:p-4 md:p-8 lg:p-10 rounded-2xl h-fit items-center text-center w-fit justify-center">
                        <h3 class="font-semibold text-fuente text-xl">{&translations["detail_store_filter_heading"]}</h3>
                        <div class="flex flex-row gap-3 lg:flex-col">
                            <p onclick={onclick_price_filter}
                                class="text-fuente p-2 font-light text-lg border-b border-r-fuente lg:border-r-0 lg:border-b-fuente select-none cursor-pointer">
                                {&translations["detail_store_filter_price"]}
                            </p>
                            <p onclick={onclick_brand_filter}
                                class="text-fuente p-2 font-light text-lg select-none cursor-pointer">
                                {&translations["detail_store_filter_brand"]}
                            </p>
                        </div>
                    </aside>

                    <div class="flex-1 grid lg:grid-cols-2 xl:grid-cols-3 gap-5 place-items-center">
                        {all_products.iter().map(|product| {
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
