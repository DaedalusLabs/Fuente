use crate::{
    contexts::{CartAction, CartStore, CommerceDataStore, ConsumerDataStore},
    router::ConsumerRoute,
};

use fuente::{contexts::LanguageConfigsStore, mass::{AppLink, Toast, ToastAction, ToastContext, ToastType}, models::ProductItem};
use lucide_yew::{ArrowLeft, ShoppingCart};
use nostr_minions::key_manager::NostrIdStore;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[derive(Clone, PartialEq, Properties)]
pub struct CommercePageProps {
    pub commerce_id: String,
}

#[derive(Clone, PartialEq, Copy)]
pub enum ProductFilter {
    Price(bool),
    Brand(bool),
}

#[function_component(CommercePage)]
pub fn commerce_page_template(props: &CommercePageProps) -> Html {
    let CommercePageProps { commerce_id } = props;
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();

    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    let commerce_profile = commerce_ctx
        .find_commerce_by_id(commerce_id)
        .expect("No commerce found");
    let products = commerce_ctx.find_product_list_by_id(commerce_id);
    let commerce_profile = commerce_profile.profile();

    let product_handle = use_state(|| None::<ProductItem>);
    let product_filter = use_state(|| Option::<ProductFilter>::None);
    let onclick_brand_filter = {
        let product_filter = product_filter.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            match product_filter.as_ref() {
                Some(ProductFilter::Price(_)) => {
                    product_filter.set(Some(ProductFilter::Brand(true)))
                }
                Some(ProductFilter::Brand(forward)) => {
                    product_filter.set(Some(ProductFilter::Brand(!forward)))
                }
                None => product_filter.set(Some(ProductFilter::Brand(true))),
            }
        })
    };
    let onclick_price_filter = {
        let product_filter = product_filter.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            match product_filter.as_ref() {
                Some(ProductFilter::Brand(_)) => {
                    product_filter.set(Some(ProductFilter::Price(true)))
                }
                Some(ProductFilter::Price(forward)) => {
                    product_filter.set(Some(ProductFilter::Price(!forward)))
                }
                None => product_filter.set(Some(ProductFilter::Price(true))),
            }
        })
    };
    let mut all_products = if let Some(products) = products {
        products
            .menu()
            .categories()
            .iter()
            .flat_map(|category| {
                category
                    .products()
                    .iter()
                    .map(|product| product.clone())
                    .collect::<Vec<ProductItem>>()
            })
            .collect::<Vec<ProductItem>>()
    } else {
        vec![]
    };
    match product_filter.as_ref() {
        Some(ProductFilter::Price(forward)) => {
            all_products.sort_by(|a, b| match forward {
                true => a.price().partial_cmp(&b.price()).unwrap(),
                false => b.price().partial_cmp(&a.price()).unwrap(),
            });
        }
        Some(ProductFilter::Brand(forward)) => {
            all_products.sort_by(|a, b| match forward {
                true => a.name().partial_cmp(&b.name()).unwrap(),
                false => b.name().partial_cmp(&a.name()).unwrap(),
            });
        }
        None => {}
    }
    match product_handle.as_ref() {
        Some(product) => html! {
            <ProductPage product={product.clone()} commerce_id={commerce_id.clone()} product_handle={product_handle.clone()} />
        },
        None => html! {
            <main class="flex flex-col w-full mx-auto">
                <div class="flex flex-col lg:flex-row justify-between items-center container mx-auto gap-4 py-5">
                    <h1 class="text-2xl font-mplus lg:text-6xl text-fuente tracking-tighter font-bold text-center line-clamp-2">
                        {&commerce_profile.name}
                    </h1>

                    <div class="relative flex items-center justify-center lg:justify-start gap-10 md:gap-0 bg-fuente rounded-2xl pr-5 w-full h-full overflow-hidden max-h-64 min-h-24">
                        <img
                            src={if commerce_profile.banner_url.is_empty() {
                                "/public/assets/img/company.png".to_string()
                            } else {
                                commerce_profile.banner_url.clone()
                            }}
                            alt={commerce_profile.name.clone()}
                            class="absolute inset-0 w-full h-full object-cover object-center"
                        />
                    </div>
                </div>

                <div class="flex-grow flex flex-col lg:flex-row overflow-hidden container mx-auto mt-5 gap-5">
                    <aside class="flex-shrink-0 overflow-auto no-scrollbar items-start justify-center flex">
                        <div class="flex flex-row lg:flex-col gap-3 bg-gray-100 items-center rounded-2xl p-2 lg:p-5 lg:w-full">
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
                        </div>
                    </aside>

                    <div class="flex-grow overflow-hidden w-full">
                        <div class="h-full overflow-auto rounded-xl no-scrollbar relative">
                            <div class="grid lg:grid-cols-2 xl:grid-cols-3 gap-5 overflow-y-auto">
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
                    </div>
                </div>
            </main>
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
    let auth_context = use_context::<NostrIdStore>().expect("No auth context found");
    let consumer_ctx = use_context::<ConsumerDataStore>().expect("No commerce context found");
    let has_address_and_profile = {
        consumer_ctx.get_default_address().is_some() && consumer_ctx.get_profile().is_some()
    };
    let is_logged_in = auth_context.get_identity().is_some();
    let navigation = use_navigator().expect("No navigation found");
    let show_warning = use_state(|| false);
    let onclick = {
        let commerce_id = commerce_id.clone();
        let cart_ctx = cart_ctx.clone();
        let show_warning = show_warning.clone();
        let product = product.clone();
        let toast_ctx = use_context::<ToastContext>().expect("No toast context");
        
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if !is_logged_in {
                navigation.push(&ConsumerRoute::Login);
                return;
            }
            if !has_address_and_profile {
                navigation.push(&ConsumerRoute::Register);
                return;
            }
            if !cart_ctx.can_add_from_business(&commerce_id) {
                show_warning.set(true);
                toast_ctx.dispatch(ToastAction::Show(Toast {
                    message: "Can't add items from different stores".into(),
                    toast_type: ToastType::Error,
                }));
                return;
            }
            cart_ctx.dispatch(CartAction::AddProduct(
                product.clone(),
                commerce_id.to_string(),
            ));
            toast_ctx.dispatch(ToastAction::Show(Toast {
                message: format!("{} added to cart", product.name()),
                toast_type: ToastType::Success,
            }));
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
        <div class="border-2 border-fuente rounded-2xl p-2 max-w-72 m-auto min-h-96 max-h-96 h-full overflow-hidden">
            <div class="flex flex-1 flex-col justify-center h-full">
                <div class="relative">
                    <img onclick={image_onclick.clone()} src={product.image_url()} alt="Favorites Image"
                    class="hidden lg:block object-contain w-full max-h-52 mx-auto bg-gray-100 rounded-2xl" />
                    <img onclick={image_onclick} src={product.thumbnail_url()} alt="Favorites Image"
                    class="block lg:hidden object-contain w-full max-h-24 mx-auto bg-gray-100 rounded-2xl" />
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
        <main class="flex flex-col h-screen overflow-hidden container mx-auto">
            <button onclick={back_to_store}
                class="flex gap-3">
                <ArrowLeft class="w-8 h-8 text-fuente" />
                <p class="text-fuente text-lg font-semibold">{"Back to store"}</p>
            </button>

            <div class="flex flex-col lg:flex-row overflow-hidden mt-5">
                <div class="grid grid-cols-1 lg:grid-cols-2 items-center overflow-y-auto w-full gap-4">
                    <div class="grid grid-cols-1 lg:grid-cols-2 place-items-center gap-2">
                        <img src={product.image_url()} alt={product.name()} class="hidden lg:block bg-gray-100 rounded-2xl w-5/6 object-contain h-full max-h-96" />

                        <div class="flex flex-col gap-4 justify-between">
                            <img src={product.thumbnail_url()} alt="Sneaker Product" class="w-40 bg-gray-100 rounded-2xl block object-contain flex-1" />
                        </div>
                    </div>

                    <div>
                        <h2 class="text-gray-500 text-2xl font-bold">{product.name()}</h2>
                        <p class="font-light text-gray-500 text-xl mt-3 text-xs sm:text-sm md:text-lg line-clamp-3">{product.details()}</p>
                        <p class="font-bold text-gray-500 uppercase text-2xl">{format!("{}", product.sku())}</p>
                        // <div class="flex items-center gap-1 mt-10">
                        //     <svg baseProfile="tiny" version="1.2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 ">
                        //         <path d="m9.362 9.158-5.268.584c-.19.023-.358.15-.421.343s0 .394.14.521c1.566 1.429 3.919 3.569 3.919 3.569-.002 0-.646 3.113-1.074 5.19a.496.496 0 0 0 .734.534c1.844-1.048 4.606-2.624 4.606-2.624l4.604 2.625c.168.092.378.09.541-.029a.5.5 0 0 0 .195-.505l-1.071-5.191 3.919-3.566a.499.499 0 0 0-.28-.865c-2.108-.236-5.269-.586-5.269-.586l-2.183-4.83a.499.499 0 0 0-.909 0l-2.183 4.83z" fill="#4167e8" class="fill-000000"></path>
                        //     </svg>
                        //     <p class="text-gray-500 text-2xl">{"5.0 (30 reviews)"}</p>
                        // </div>

                        <div class="flex flex-row lg:flex-col mt-5 lg:mt-10 gap-5">
                            <p class="text-3xl md:text-4xl lg:text-5xl font-bold text-fuente">{format!("SRD {}", product.price())}</p>
                            <button onclick={add_cart}
                                class="bg-fuente-orange text-white p-2  lg:py-4 lg:px-10 rounded-full flex items-center justify-center gap-2 flex-1">
                                <ShoppingCart class="w-8 h-8" />
                                <p class="hidden md:block font-semibold text-xl text-center">{"Shop Now"}</p>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
            <section class="bg-gray-100 p-4 rounded-2xl mb-5 mx-auto space-y-7">
                <p class="text-gray-500 leading-6 text-xs sm:text-sm md:text-lg line-clamp-2">{product.description()}</p>
            </section>

        </main>
    }
}

#[function_component(AllCommercesPage)]
pub fn settings_template() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    let businesses = commerce_ctx.commerces();
    html! {
        <main class="flex flex-col h-full overflow-hidden container mx-auto">
            <div class="flex flex-col lg:flex-row justify-between items-center my-5">
                <h1 class="text-3xl font-mplus lg:text-6xl text-nowrap text-fuente tracking-tighter font-bold text-center">
                    {&translations["stores_heading"]}
                </h1>
            </div>

            <div class="flex-1 w-full flex flex-col lg:flex-row overflow-hidden mt-2">
                <div class="w-full grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 overflow-y-auto">
                    {businesses.iter().map(|profile| {
                        let commerce_data = profile.profile().clone();
                        let commerce_id = profile.id().to_string();
                        html! {
                            <AppLink<ConsumerRoute>
                                class="border-2 border-fuente rounded-3xl block object-contain bg-white overflow-clip p-2 h-80"
                                selected_class=""
                                route={ConsumerRoute::Commerce { commerce_id: commerce_id.clone() }}
                            >
                            <div class="flex flex-col md:flex-row items-center justify-center gap-5 w-full h-fit">
                                <img src={commerce_data.logo_url.clone()} alt="Company Image" class="w-36" />
                                <div class="space-y-2 flex flex-col items-center">
                                    <h3 class="text-gray-500 text-lg font-bold tracking-wide uppercase">{&commerce_data.name}</h3>
                                    <p class="text-gray-500 font-light text-md line-clamp-3">{&commerce_data.description}</p>
                                    // <div class="flex items-center gap-2">
                                    //     <Star class="w-6 h-6 text-fuente" />
                                    //     <p class="text-gray-500 font-light">{"5.0 Delivery on time"}</p>
                                    // </div>
                                </div>
                            </div>
                            </AppLink<ConsumerRoute>>
                        }
                    }).collect::<Html>()}
                </div>
            </div>
        </main>
    }
}
