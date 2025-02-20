use yew::prelude::*;

use crate::contexts::LanguageConfigsStore;
use crate::models::{CommerceProfile, ParticipantRating};

#[derive(Clone, Properties, PartialEq)]
pub struct CommerceProfileProps {
    pub commerce_data: CommerceProfile,
    #[prop_or_default]
    pub rating: Option<ParticipantRating>,
}

#[function_component(CommerceProfileCard)]
pub fn business_card(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps {
        commerce_data,
        rating: _,
    } = props;
    let logo_url = if commerce_data.logo_url.is_empty() {
        "/public/assets/img/company.png".to_string()
    } else {
        commerce_data.logo_url.clone()
    };
    html! {
        <div class="flex flex-col items-center">
            <div class="w-full aspect-square overflow-hidden rounded-lg -m-2">
                <img 
                    src={logo_url} 
                    alt={commerce_data.name.clone()}
                    class="w-full h-full object-cover object-center"
                />
            </div>
        </div>
    }
}

#[function_component(CommerceProfileDetails)]
pub fn business_details(props: &CommerceProfileProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("LanguageStore not found");
    let translations = language_ctx.translations();
    let CommerceProfileProps {
        commerce_data,
        rating: _,
    } = props;
    html! {
        <section class="lg:mt-5 space-y-3 border-t border-t-gray-400 md:border-t-0 py-3 w-full">
            <h3 class="text-gray-500 font-light text-lg">{&translations["stores_settings_option_information"]}</h3>
            <p class="text-gray-500 font-bold text-lg">{&commerce_data.name}</p>
            <div class="w-full flex flex-col gap-2 justify-between">
                <div class="flex gap-2 items-center">
                    <p class="text-gray-500 font-bold text-lg">{&translations["checkout_client_information_heading_email"]}</p>
                    <p class="text-gray-500 flex-1 text-right">{&commerce_data.web}</p>
                </div>

                <div class="flex gap-2 items-center">
                    <p class="text-gray-500 font-bold text-lg">{&translations["checkout_client_information_heading_phone"]}</p>
                    <p class="text-gray-500 flex-1 text-right">{&commerce_data.telephone}</p>
                </div>
            </div>
        </section>
    }
}

#[function_component(CommerceProfileAddressDetails)]
pub fn business_address_details(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps {
        commerce_data,
        rating: _,
    } = props;
    html! {
        <section class="space-y-3 py-3 w-full md:col-start-1 md:col-end-3">
            <span class="text-neutral-400 line-clamp-3 text-center">{commerce_data.lookup.display_name()}</span>
        </section>
    }
}
