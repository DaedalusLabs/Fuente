use lucide_yew::{ArrowLeft, ArrowRight, Headset, ShieldCheck, SquarePen, Truck};
use nostro2::notes::NostrNote;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::{
    contexts::LanguageConfigsStore,
    models::{DriverProfile, OrderInvoiceState, OrderStatus},
};

#[derive(Clone, PartialEq, Properties)]
pub struct SettingsSideBarBrops {
    pub options: Vec<(String, Callback<MouseEvent>, bool)>,
}

#[function_component(SettingsSideBar)]
pub fn settings_sidebar(props: &SettingsSideBarBrops) -> Html {
    let SettingsSideBarBrops { options } = props;
    let selected_class = classes!("bg-fuente", "text-white");
    let unselected_class = classes!("bg-gray-100", "text-gray-500");
    html! {
        <aside class="flex flex-col gap-3">
            {for options.iter().map(|(name, onclick, selected)| {
                html! {
                    <button
                        type="button"
                        class={classes!(
                            "py-4",
                            "px-8",
                            "rounded-2xl",
                            "font-bold",
                            "text-lg",
                            if *selected { selected_class.clone() } else { unselected_class.clone() }
                        )}
                        {onclick}>
                        {name.as_str()}
                    </button>
                }
            })}
        </aside>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsContentProps {
    pub children: Children,
    pub edit_button: Option<Html>,
}

#[function_component(SettingsContent)]
pub fn settings_content(props: &SettingsContentProps) -> Html {
    let SettingsContentProps {
        children,
        edit_button,
    } = props;
    html! {
        <div class="border border-fuente rounded-xl flex items-start justify-between flex-1 gap-5" style="padding: 40px 50px 40px 80px;">
            <div class="w-full">
                {children}
            </div>
            {edit_button.clone().unwrap_or_default()}

        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsButtonContentProps {
    pub onclick: Callback<MouseEvent>,
}

#[function_component(SettingsContentButton)]
pub fn settings_content_button(props: &SettingsButtonContentProps) -> Html {
    let SettingsButtonContentProps { onclick } = props;
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    html! {
        <button type="button" class="flex gap-4 tracking-wide" {onclick}>
            <span class="text-fuente font-bold text-xl">{&translations["profile_address_edit_button"]}</span>
            <SquarePen class="w-6 h-6 text-fuente" />
        </button>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct SettingsOptionsButtonsProps {
    pub options: Vec<Html>,
}

#[function_component(SettingsOptionsButtons)]
pub fn settings_options_buttons(props: &SettingsOptionsButtonsProps) -> Html {
    let SettingsOptionsButtonsProps { options } = props;
    html! {
        <div class="flex items-center gap-4">
            {for options.iter().map(|name| {
                html! {
                    {name.clone()}
                }
            })}
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct SettingsPageTemplateProps {
    pub children: Children,
    pub options: Vec<Html>,
    pub heading: String,
    pub content_button: Option<Html>,
    pub sidebar_options: Vec<(String, Callback<MouseEvent>, bool)>,
}

#[function_component(SettingsPageTemplate)]
pub fn settings_template(props: &SettingsPageTemplateProps) -> Html {
    let SettingsPageTemplateProps {
        children,
        options,
        heading,
        sidebar_options,
        content_button,
    } = props.clone();
    html! {
        <>
    <div class="container mx-auto py-10 flex justify-between">
        <h1 class="text-fuente text-6xl font-bold tracking-tighter">{&heading}</h1>
        <SettingsOptionsButtons {options} />
    </div>

    <main class="container mx-auto">
        <div class="flex gap-10">
            <SettingsSideBar options={sidebar_options} />
            <SettingsContent edit_button={content_button} >
                <>
                    {children}
                </>
            </SettingsContent>
        </div>
    </main>
    </>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct LoginPageProps {
    pub children: Children,
    pub heading: String,
    pub sub_heading: String,
    pub title: String,
}

#[function_component(LoginPageTemplate)]
pub fn login_template(props: &LoginPageProps) -> Html {
    let LoginPageProps {
        heading,
        sub_heading,
        title,
        children,
    } = props;
    html! {
        <main class="grid grid-rows-[3fr_1fr] lg:flex min-h-screen">
            <div class="hidden bg-fuente min-h-screen overflow-hidden lg:overflow-visible lg:bg-white lg:bg-logo lg:bg-no-repeat lg:bg-[length:200px_75px] lg:mt-10 lg:ml-16 lg:flex lg:justify-end lg:items-center lg:flex-1">
                <h2 class="hidden lg:flex text-fuente text-[135px] lg:text-[175px] font-bold -rotate-90 -mr-24 tracking-tighter lg:tracking-[-1rem]">
                    {&heading}
                </h2>
            </div>

            <div class="bg-fuente lg:flex lg:items-center lg:flex-auto lg:h-auto lg:min-h-screen lg:overflow-visible relative lg:static">
                <h2 class="hidden lg:flex text-white text-[135px] lg:text-[175px] font-bold -rotate-90 -ml-[5.4rem] lg:-ml-[7.9rem] tracking-[-1rem] -mb-16">
                    {&sub_heading}
                </h2>
                <div class="container mx-auto pt-10">
                    <div class="max-w-[400px] lg:max-w-[360px] mx-auto xl:ml-44 lg:mb-48">
                        <p class="text-5xl lg:text-6xl text-white font-bold -mb-2 relative z-10 text-right tracking-tighter mr-4">
                            {&title}
                        </p>
                        {children}
                    </div>
                </div>

                <h2 class="text-white text-[135px] font-bold tracking-[-1rem] text-center lg:hidden absolute -bottom-12 left-0 w-full">{&heading}</h2>
            </div>
            <div class="lg:hidden bg-white">
                <h2 class="text-fuente text-[135px] font-bold tracking-[-1rem] text-center -mt-5">{&sub_heading}</h2>
            </div>
        </main>
    }
}
#[function_component(FuenteBitcoinBanner)]
pub fn bitcoin_banner() -> Html {
    html! {
    <div class="container mx-auto grid gap-5 lg:gap-0 grid-cols-2 sm:grid-cols-[3fr_1fr] place-items-center">
        <div class="bg-orange-400 w-full mt-10 rounded-2xl h-fit lg:max-h-52">
            <div class="flex items-center">
                <svg viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg" class="w-80 -rotate-12 -mt-5 lg:-mt-14">
                    <path fill="none" d="M0 0h256v256H0z"></path><path d="M184 184H69.8L41.9 30.6a8 8 0 0 0-7.8-6.6H16" fill="none" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></path><circle cx="80" cy="204" fill="none" r="20" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></circle><circle cx="184" cy="204" fill="none" r="20" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></circle><path d="M62.5 144h125.6a15.9 15.9 0 0 0 15.7-13.1L216 64H48" fill="none" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></path>
                </svg>
                <h2 class="text-3xl lg:text-5xl tracking-tighter text-white font-semibold max-w-[500px] lg:-mt-14 mx-auto">{"Shop now, track your package and pay with"}</h2>
            </div>
        </div>

        <img src="/templates/img/bitcoin.png" alt="Bitcoin Logo" class="hidden lg:flex" />
        <svg height="512px" id="svg2" preserveAspectRatio="xMidYMid" version="1.1" viewBox="0 0 1 1" width="512px" xmlns="http://www.w3.org/2000/svg" class="lg:hidden w-1/2 h-full">
            <defs id="defs4"><filter color-interpolation-filters="sRGB" id="_drop-shadow"><feGaussianBlur id="feGaussianBlur7" in="SourceAlpha" result="blur-out" stdDeviation="1"/><feBlend id="feBlend9" in="SourceGraphic" in2="blur-out" mode="normal"/></filter><linearGradient id="coin-gradient" x1="0%" x2="0%" y1="0%" y2="100%"><stop id="stop12" offset="0%" style="stop-color:#f9aa4b"/><stop id="stop14" offset="100%" style="stop-color:#f7931a"/></linearGradient></defs><g id="g16" transform="scale(0.015625)"><path d="m 63.0359,39.741 c -4.274,17.143 -21.637,27.576 -38.782,23.301 -17.138,-4.274 -27.571,-21.638 -23.295,-38.78 4.272,-17.145 21.635,-27.579 38.775,-23.305 17.144,4.274 27.576,21.64 23.302,38.784 z" id="coin" style="fill:url(#coin-gradient)"/><path d="m 46.1009,27.441 c 0.637,-4.258 -2.605,-6.547 -7.038,-8.074 l 1.438,-5.768 -3.511,-0.875 -1.4,5.616 c -0.923,-0.23 -1.871,-0.447 -2.813,-0.662 l 1.41,-5.653 -3.509,-0.875 -1.439,5.766 c -0.764,-0.174 -1.514,-0.346 -2.242,-0.527 l 0.004,-0.018 -4.842,-1.209 -0.934,3.75 c 0,0 2.605,0.597 2.55,0.634 1.422,0.355 1.679,1.296 1.636,2.042 l -1.638,6.571 c 0.098,0.025 0.225,0.061 0.365,0.117 -0.117,-0.029 -0.242,-0.061 -0.371,-0.092 l -2.296,9.205 c -0.174,0.432 -0.615,1.08 -1.609,0.834 0.035,0.051 -2.552,-0.637 -2.552,-0.637 l -1.743,4.019 4.569,1.139 c 0.85,0.213 1.683,0.436 2.503,0.646 l -1.453,5.834 3.507,0.875 1.439,-5.772 c 0.958,0.26 1.888,0.5 2.798,0.726 l -1.434,5.745 3.511,0.875 1.453,-5.823 c 5.987,1.133 10.489,0.676 12.384,-4.739 1.527,-4.36 -0.076,-6.875 -3.226,-8.515 2.294,-0.529 4.022,-2.038 4.483,-5.155 z m -8.022,11.249 c -1.085,4.36 -8.426,2.003 -10.806,1.412 l 1.928,-7.729 c 2.38,0.594 10.012,1.77 8.878,6.317 z m 1.086,-11.312 c -0.99,3.966 -7.1,1.951 -9.082,1.457 l 1.748,-7.01 c 1.982,0.494 8.365,1.416 7.334,5.553 z" id="symbol" style="fill:#ffffff"/></g>
        </svg>
    </div>
    }
}
#[function_component(FuenteHotCategories)]
pub fn categories_banner() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    html! {
    <main class="container mx-auto lg:mt-20 flex flex-col lg:grid lg:grid-cols-[1fr_3fr] lg:gap-5">
        <div class="bg-fuente rounded-2xl p-5 flex flex-col lg:justify-between lg:relative">
            <div class="flex justify-between items-center lg:mb-4">
                <h2 class="text-white text-4xl font-semibold tracking-tighter">{&translations["home_stores"]}</h2>
                <ArrowRight class="w-12 h-12 text-white rounded-full border-4 border-white" />
            </div>

            <img src="/templates/img/store.png" alt="Store Image" class="object-contain w-64 mx-auto lg:absolute lg:bottom-0 lg:right-8" />
        </div>
        <div class="overflow-x-auto whitespace-nowrap mt-10 lg:mt-0">
            <div class="flex justify-between items-center">
                <h2 class="text-fuente text-5xl font-bold tracking-tighter">{&translations["home_hot_categories_heading"]}</h2>
                <ArrowRight class="w-12 h-12 text-fuente rounded-full border-4 border-fuente" />
            </div>

            <div class=" mt-10">
                <div class="grid grid-flow-col auto-cols-max gap-4">
                    <div class="bg-fuente-light rounded-2xl flex items-center w-80">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{&translations["home_hot_categories_books"]}</p>
                    </div>
                    <img src="/templates/img/iphone.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/sneaker_1.png" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/yumi.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <div class="bg-fuente-light rounded-2xl w-80 flex items-center">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{&translations["home_hot_categories_tech"]}</p>
                    </div>
                    <img src="/templates/img/iphone.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/sneaker_1.png" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/yumi.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                </div>

                <div class="grid grid-flow-col auto-cols-max gap-4 mt-5">
                    <img src="/templates/img/ninja.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <div class="bg-fuente-light rounded-2xl flex items-center w-80">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{&translations["home_hot_categories_movies"]}</p>
                    </div>
                    <img src="/templates/img/candy.jpg" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/whey.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/ninja.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <div class="bg-fuente-light rounded-2xl w-80 flex items-center">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{&translations["home_hot_categories_music"]}</p>
                    </div>
                    <img src="/templates/img/candy.jpg" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/whey.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                </div>
            </div>
        </div>
    </main>
    }
}
#[function_component(FuenteSalesPitch)]
pub fn sales_pitch() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    html! {
    <div class="flex flex-col justify-center lg:flex-row lg:justify-between items-center lg:relative bg-sky-200 rounded-2xl px-10 py-5 container mx-auto mt-7">
        <div class="flex">
            <img src="templates/img/sneaker_1.png" alt="Sneaker Product" class="lg:absolute object-contain w-32 lg:w-40 mt-10 xl:mt-4 xl:w-52 top-0 left-0 z-10" />
            <img src="templates/img/sneaker_2.png" alt="Sneaker Product" class="lg:absolute object-contain w-32 lg:w-40 mt-10 xl:mt-4 xl:w-52 -top-10 left-28 z-20" />
            <img src="templates/img/sneaker_3.png" alt="Sneaker Product" class="lg:absolute object-contain w-32 lg:w-40 mt-10 xl:mt-4 xl:w-64 lg:top-0 lg:left-56 z-30 hidden lg:flex"/>
        </div>

        <div class="mx-auto lg:mx-0 lg:ml-auto">
            <h2 class="text-5xl text-fuente tracking-tighter font-semibold max-w-[500px] text-center lg:text-left">{&translations["sale_products_heading"]}</h2>
            <div class="flex justify-center lg:justify-start">
                <button class="text-fuente-forms bg-fuente-buttons py-3 px-10 rounded-full font-semibold mt-5">{&translations["sale_products_button"]}</button>
            </div>
        </div>
    </div>
    }
}

#[function_component(FuenteBenefits)]
pub fn benefits() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    html! {
    <section class="mt-5 container mx-auto">
        <div class="grid lg:grid-cols-3 gap-5 bg-gray-100 p-12 rounded-2xl place-content-center lg:place-items-center">
            <div class="flex items-center gap-5">
                <Headset class="h-28 w-28 bg-fuente rounded-2xl p-3 text-white" />
                <div>
                    <h3 class="text-fuente-dark text-xl font-semibold">{&translations["benefits_support_heading"]}</h3>
                    <p class="text-lg text-fuente-dark">{&translations["benefits_support_text"]}</p>
                    <p class="text-lg text-fuente-dark">{&translations["benefits_support_text_description"]}</p>
                </div>
            </div>
            <div class="flex items-center gap-5">
                <Truck class="h-28 w-28 bg-fuente rounded-2xl p-3 text-white" />
                <div>
                    <h3 class="text-fuente-dark text-xl font-semibold">{&translations["benefits_track_heading"]}</h3>
                    <p class="text-lg text-fuente-dark">{&translations["benefits_track_text"]}</p>
                    <p class="text-lg text-fuente-dark">{&translations["benefits_track_text_description"]}</p>
                </div>
            </div>
            <div class="flex items-center gap-5">
                <ShieldCheck class="h-28 w-28 bg-fuente rounded-2xl p-3 text-white" />
                <div>
                    <h3 class="text-fuente-dark text-xl font-semibold">{&translations["benefits_secure_heading"]}</h3>
                    <p class="text-lg text-fuente-dark">{&translations["benefits_secure_text"]}</p>
                    <p class="text-lg text-fuente-dark">{&translations["benefits_secure_text_description"]}</p>
                </div>
            </div>
        </div>
    </section>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct OrderHistoryTemplateProps {
    pub orders: Vec<OrderInvoiceState>,
}

#[function_component(OrderHistoryTemplate)]
pub fn order_history_template(props: &OrderHistoryTemplateProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let filter_state = use_state(|| OrderStatus::Completed);
    let selected_order = use_state(|| None::<String>);
    let selected_class = classes!("bg-fuente", "text-white");
    let unselected_class = classes!("bg-gray-100", "text-gray-400");
    let back_button = classes!(
        "flex",
        "gap-2",
        "items-center",
        "justify-center",
        "bg-fuente",
        "text-white",
        "text-center",
        "font-bold",
        "text-lg",
        "rounded-2xl",
        "py-4",
        "w-56",
    );
    let completed_filter_button_class = classes!(
        "text-center",
        "font-bold",
        "text-lg",
        "rounded-2xl",
        "py-4",
        "w-56",
        if *filter_state == OrderStatus::Completed {
            selected_class.clone()
        } else {
            unselected_class.clone()
        }
    );
    let set_completed_filter = {
        let filter_state = filter_state.clone();
        Callback::from(move |_| filter_state.set(OrderStatus::Completed))
    };
    let canceled_filter_button_class = classes!(
        "text-center",
        "font-bold",
        "text-lg",
        "rounded-2xl",
        "py-4",
        "w-56",
        if *filter_state == OrderStatus::Canceled {
            selected_class.clone()
        } else {
            unselected_class.clone()
        }
    );
    let set_canceled_filter = {
        let filter_state = filter_state.clone();
        Callback::from(move |_| filter_state.set(OrderStatus::Canceled))
    };
    let order_handler = selected_order.clone();
    let onclick_order = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        let order_id = e.target_dyn_into::<HtmlElement>().unwrap().id();
        if order_id.is_empty() {
            return;
        }
        order_handler.set(Some(order_id));
    });

    let mut filtered_orders = (props.orders)
        .iter()
        .filter(|order| order.order_status == *filter_state)
        .cloned()
        .collect::<Vec<_>>();
    filtered_orders.sort_by(|a, b| b.order_timestamp().cmp(&a.order_timestamp()));

    if filtered_orders.is_empty() {
        return html! {};
    }
    let unselect_order = {
        let selected = selected_order.clone();
        Callback::from(move |_| selected.set(None))
    };
    html! {
        <main class="mt-16 container mx-auto">
            <div class="w-full flex items-center justify-between">
                <h1 class="text-6xl font-bold text-fuente tracking-tighter">{&translations["profile_address_button_orders"]}</h1>
                {match *selected_order {
                    Some(_) => html! {
                        <div class="mt-5 flex gap-4">
                            <button
                                onclick={unselect_order}
                                class={back_button}>
                                <ArrowLeft class="w-6 h-6 text-white" />
                                {&translations["store_orders_history_back"]}
                            </button>
                        </div>
                    },
                    None => html! {
                        <div class="mt-5 flex gap-4">
                            <button class={completed_filter_button_class} onclick={set_completed_filter}>
                                {&translations["store_orders_history_completed"]}</button>
                            <button class={canceled_filter_button_class} onclick={set_canceled_filter}>
                                {&translations["store_orders_history_canceled"]}
                            </button>
                        </div>
                    },
                }}
            </div>

            {match selected_order.as_ref() {
                Some(order) => {
                    let order = (*filtered_orders).iter().find(|o| o.order_id() == *order).expect("Order not found");
                    html! {
                        <OrderDetails
                            order={order.clone()}
                        />
                    }
                },
                None => html! {
                    <table class="table-auto w-full border-collapse mt-5">
                        <thead>
                            <tr>
                                <th class="text-fuente-dark font-semibold text-center pb-5 px-5 text-xl">{&translations["packages_track_table_heading_order"]}</th>
                                <th class="text-fuente-dark font-semibold text-center pb-5 px-5 text-xl">{&translations["packages_track_table_heading_date"]}</th>
                                <th class="text-fuente-dark font-semibold text-center pb-5 px-5 text-xl">{&translations["packages_track_table_heading_driver"]}</th>
                            </tr>
                        </thead>
                        <tbody class="bg-gray-100 space-y-2">
                            {filtered_orders.iter().map(|order| {
                                let driver_profile = DriverProfile::try_from(order.courier.as_ref().unwrap_or(&NostrNote::default()));
                                let driver_name = driver_profile.map(|profile| profile.nickname()).unwrap_or_default();
                                let driver_id = order.courier.as_ref().unwrap_or(&NostrNote::default()).pubkey.clone();
                                html! {
                                    <tr onclick={onclick_order.clone()} id={order.order_id()} class="cursor-pointer hover:bg-gray-200">
                                        <td id={order.order_id()} class="text-center text-xl font-semibold text-fuente-dark">{format!("#{}", &order.order_id()[..12])}</td>
                                        <td id={order.order_id()} class="text-center text-xl font-semibold text-fuente-dark">{format!("{} - {}", order.locale_date(), order.locale_time())}</td>
                                        {if driver_name.is_empty() {
                                            html! {
                                                <td id={order.order_id()} class="text-center text-fuente-dark font-semibold text-xl">{"-"}</td>
                                            }
                                        } else {
                                            html! {
                                                <td id={order.order_id()} class="text-center text-fuente-dark font-semibold text-xl space-y-2">
                                                    <p id={order.order_id()} class="text-xl text-gray-500 font-bold">{driver_name}</p>
                                                    <p id={order.order_id()} class="text-gray-500 font-light">{format!("{}", &driver_id[..12])}</p>
                                                </td>
                                            }
                                        }}
                                    </tr>
                                }
                            }).collect::<Html>()}
                        </tbody>
                    </table>
                },
            }}

        </main>
    }
}

#[derive(Properties, Clone, PartialEq)]
struct OrderDetailsProps {
    order: OrderInvoiceState,
}

#[function_component(OrderDetails)]
fn order_details(props: &OrderDetailsProps) -> Html {
    let order_req = props.order.get_order_request();
    let products = order_req.products.counted_products();
    let profile = order_req.profile;

    html! {
        <div class="flex flex-col w-full h-full">
            <div class="flex items-center gap-4 mb-6 p-6">
                <h2 class="text-2xl font-semibold text-fuente-dark">
                    {format!("Order Details #{}", &props.order.order_id()[..8])}
                </h2>
            </div>

            <div class="grid grid-cols-2 gap-8 p-6">
                <div class="space-y-6">
                <div class="space-y-2">
                <h3 class="font-medium text-gray-500">{"Customer Information"}</h3>
                    <div class="space-y-1">
                        <p>{"Name: "}{profile.nickname}</p>
                        <p>{"Phone: "}{profile.telephone}</p>
                        <p>{"Email: "}{profile.email}</p>
                    </div>
            </div>

                    <div class="space-y-2">
                        <h3 class="font-medium text-gray-500">{"Delivery Address"}</h3>
                        <p class="text-sm">{order_req.address.lookup().display_name()}</p>
                    </div>

                    <div class="space-y-2">
                        <h3 class="font-medium text-gray-500">{"Order Status"}</h3>
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
                </div>

                <div class="space-y-4">
                    <h3 class="font-medium text-gray-500">{"Order Items"}</h3>
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
