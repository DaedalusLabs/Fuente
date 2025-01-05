use fuente::contexts::LanguageConfigsStore;
use yew::prelude::*;

use crate::ServerConfigsStore;

#[function_component(HomePage)]
pub fn exchange_rate_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let exchange_rate = config_ctx.get_exchange_rate();
    let commerce_white_list = config_ctx.get_commerce_whitelist().len();
    let driver_white_list = config_ctx.get_couriers_whitelist().len();
    html! {
        <>
        <div class="container mx-auto lg:py-10 flex flex-col lg:flex-row items-center lg:justify-between">
            <h3 class="text-fuente text-4xl pb-10 lg:pb-0 text-center lg:text-left lg:text-6xl font-bold tracking-tighter">
                {&translations["admin_settings_title_home"]}
            </h3>
        </div>
        <main class="container mx-auto mt-10 max-h-full pb-4 overflow-y-clip no-scrollbar">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                <div class="bg-white gap-2 p-4 rounded-lg shadow flex flex-col items-center justify-center">
                    <h2 class="text-2xl font-bold text-fuente">{&translations["admin_settings_title_exchange"]}</h2>
                    <p class="text-lg">{format!("1 BTC")}</p>
                    <p class="text-lg">{format!("=")}</p>
                    <p class="text-lg">{format!("{} SRD", exchange_rate)}</p>
                </div>
                <div class="bg-white gap-2 p-4 rounded-lg shadow flex flex-col items-center justify-center">
                    <h2 class="text-2xl font-bold text-fuente">{&translations["admin_settings_commerces_registered"]}</h2>
                    <p class="text-4xl">{format!("{}", commerce_white_list)}</p>
                </div>
                <div class="bg-white gap-2 p-4 rounded-lg shadow flex flex-col items-center justify-center">
                    <h2 class="text-2xl font-bold text-fuente">{&translations["admin_settings_courier_count"]}</h2>
                    <p class="text-4xl">{format!("{}", driver_white_list)}</p>
                </div>
            </div>
        </main>
        </>
    }
}

