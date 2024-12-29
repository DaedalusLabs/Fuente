use yew::prelude::*;

use crate::models::CommerceProfile;

#[derive(Clone, Properties, PartialEq)]
pub struct CommerceProfileProps {
    pub commerce_data: CommerceProfile,
}

#[function_component(CommerceProfileCard)]
pub fn business_card(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
            <img src={commerce_data.logo_url.clone()} alt={commerce_data.name.clone()}
                class="w-full h-full min-h-96 min-w-64" />
    }
}

#[function_component(CommerceProfileDetails)]
pub fn business_details(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
        <div class="flex flex-row gap-4">
            <div class="w-16 h-16 bg-neutral-200 rounded-2xl"></div>
            <div class="flex flex-col">
                <span class="font-bold text-lg mb-1">{&commerce_data.name}</span>
                <span class="text-neutral-400">{&commerce_data.telephone}</span>
                <span class="text-neutral-400">{&commerce_data.web}</span>
                <span class="text-neutral-400">{&commerce_data.description}</span>
            </div>
        </div>
    }
}

#[function_component(CommerceProfileAddressDetails)]
pub fn business_address_details(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
        <div class="flex flex-row gap-4">
            <div class="min-h-16 min-w-16 w-16 h-16 bg-neutral-200 rounded-2xl"></div>
            <div class="flex flex-col">
                <span class="font-bold text-lg mb-1">{commerce_data.lookup.name()}</span>
                <span class="text-neutral-400">{commerce_data.lookup.display_name()}</span>
            </div>
        </div>
    }
}
