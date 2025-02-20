use fuente::contexts::LanguageConfigsStore;
use yew::prelude::*;

use crate::{PlatformStatsStore, ServerConfigsStore};

#[function_component(HomePage)]
pub fn exchange_rate_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let stats_ctx = use_context::<PlatformStatsStore>().expect("PlatformStatsStore not found");
    let commerce_white_list = config_ctx.get_commerce_whitelist().len();
    let users = stats_ctx.count_users();
    let completed_orders = stats_ctx.count_completed_orders();
    let canceled_orders = stats_ctx.count_pending_orders();
    html! {
        <main class="container mx-auto overflow-hidden">
            <div class="flex flex-col h-full">
                <div class="flex flex-row justify-between items-center p-4 lg:py-10">
                    <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-0 lg:text-6xl tracking-tighter font-bold font-mplus">
                        {&translations["admin_settings_title_home"]}
                    </h1>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-5 p-5 lg:p-0 lg:px-5">
                    <div class="rounded-2xl admin-shadow flex items-center justify-center gap-5 p-3">
                        <div>
                            <svg class="w-12 h-12" fill="none" stroke="#4167e8" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"></path>
                            </svg>
                        </div>
                        <div class="flex flex-col gap-2">
                            <p class="text-3xl text-fuente font-bold">{users}</p>
                            <p class="text-sm font-light text-gray-500">{"Active Users"}</p>
                        </div>
                    </div>

                    <div class="rounded-2xl admin-shadow flex items-center justify-center gap-5 p-3">
                        <div>
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="h-12 w-12">
                                <path d="M5.4 24h13.2a4 4 0 0 0 3.968-4.5l-1.25-10A4.005 4.005 0 0 0 17.352 6H17V5A5 5 0 0 0 7 5v1h-.352A4.005 4.005 0 0 0 2.68 9.5l-1.25 10A4 4 0 0 0 5.4 24ZM9 5a3 3 0 0 1 6 0v1H9ZM3.414 19.752l1.25-10A2 2 0 0 1 6.648 8H7v2a1 1 0 0 0 2 0V8h6v2a1 1 0 0 0 2 0V8h.352a2 2 0 0 1 1.984 1.752l1.25 10A2 2 0 0 1 18.6 22H5.4a2 2 0 0 1-1.984-2.248Z" fill="#4167e8" class="fill-232323"></path>
                            </svg>
                        </div>
                        <div class="flex flex-col gap-2">
                            <p class="text-3xl text-fuente font-bold">{commerce_white_list}</p>
                            <p class="text-sm font-light text-gray-500">{"Active Stores"}</p>
                        </div>
                    </div>

                    <div class="rounded-2xl admin-shadow flex items-center justify-center gap-5 p-3">
                        <div>
                            <svg viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg" class="h-10 w-10">
                                <g data-name="Layer 28"><path d="M16 31a15 15 0 1 1 15-15 15 15 0 0 1-15 15Zm0-28a13 13 0 1 0 13 13A13 13 0 0 0 16 3Z" fill="#84cc16" class="fill-101820"></path><path d="M13.67 22a1 1 0 0 1-.73-.32l-4.67-5a1 1 0 0 1 1.46-1.36l3.94 4.21 8.6-9.21a1 1 0 1 1 1.46 1.36l-9.33 10a1 1 0 0 1-.73.32Z" fill="#84cc16" class="fill-101820"></path></g>
                            </svg>
                        </div>
                        <div class="flex flex-col gap-2">
                            <p class="text-3xl text-lime-500 font-bold">{completed_orders}</p>
                            <p class="text-sm font-light text-gray-500">{"Completed Orders"}</p>
                        </div>
                    </div>

                    <div class="rounded-2xl admin-shadow flex items-center justify-center gap-5 p-3">
                        <div>
                            <svg viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg" class="h-12 w-12">
                                <g data-name="30-OS X"><circle cx="16" cy="16" r="15" fill="none" stroke="#ef4444" stroke-linejoin="round" stroke-width="2px" class="stroke-000000"></circle><path d="m8 8 16 16M24 8 8 24" fill="none" stroke="#ef4444" stroke-linejoin="round" stroke-width="2px" class="stroke-000000"></path></g>
                            </svg>
                        </div>
                        <div class="flex flex-col gap-2">
                            <p class="text-3xl text-red-500 font-bold">{canceled_orders}</p>
                            <p class="text-sm font-light text-gray-500">{"Canceled Orders"}</p>
                        </div>
                    </div>
                </div>
            </div>
        </main>
    }
}
