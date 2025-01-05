use yew::prelude::*;

use crate::{contexts::LanguageConfigsStore, models::DriverProfile};

#[derive(Clone, PartialEq, Properties)]
pub struct DriverDetailsProps {
    pub pubkey: String,
    pub driver: DriverProfile,
}

#[function_component(DriverDetailsComponent)]
pub fn driver_details(props: &DriverDetailsProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    html! {
    <div class="grid grid-cols-1 gap-10 h-full w-full">
        <div class="space-y-3">
            <h3 class="text-gray-500 text-2xl font-semibold">{props.driver.nickname()}</h3>

            <div class="flex flex-row xl:items-center justify-between w-full">
                <div class="flex flex-col xl:flex-row xl:items-center justify-between">
                    <p class="text-gray-500 text-lg font-bold">{&translations["admin_courier_details_phone"]}</p>
                    <p class="text-gray-500 font-light">{&props.driver.telephone()}</p>
                </div>
                <div class="flex flex-col xl:flex-row xl:items-center justify-evenly">
                    <p class="text-gray-500 text-lg font-bold">{&translations["admin_courier_details_key"]}</p>
                    <p class="text-gray-500 font-light">{&props.pubkey[..12].to_string()}</p>
                </div>
            </div>
        </div>
    </div>
    }
}
