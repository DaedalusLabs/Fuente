use yew::prelude::*;

use crate::models::commerce::CommerceProfile;

#[derive(Clone, Properties, PartialEq)]
pub struct CommerceProfileProps {
    pub commerce_data: CommerceProfile,
}

#[function_component(CommerceProfileCard)]
pub fn business_card(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
        <div class="flex flex-col justify-between rounded-3xl p-4 px-8 bg-neutral-50
                gap-8 shadow-xl items-center">
            <div class="w-32 h-32 bg-neutral-300 rounded-full -mt-12"></div>
            <div class="flex flex-col gap-1 items-center text-center">
                <h5 class="font-semibold tracking-wide">{commerce_data.name()}</h5>
                <p class="text-sm text-neutral-400">{commerce_data.description()}</p>
            </div>
        </div>
    }
}

#[function_component(CommerceProfileDetails)]
pub fn business_details(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
        <div class="flex flex-row gap-4">
            <div class="w-16 h-16 bg-neutral-200 rounded-2xl"></div>
            <div class="flex flex-col">
                <span class="font-bold text-lg mb-1">{commerce_data.name()}</span>
                <span class="text-neutral-400">{commerce_data.telephone()}</span>
                <span class="text-neutral-400">{commerce_data.web()}</span>
                <span class="text-neutral-400">{commerce_data.description()}</span>
            </div>
        </div>
    }
}

#[function_component(CommerceProfileAddressDetails)]
pub fn business_address_details(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    let lookup = commerce_data.lookup();
    html! {
        <div class="flex flex-row gap-4">
            <div class="min-h-16 min-w-16 w-16 h-16 bg-neutral-200 rounded-2xl"></div>
            <div class="flex flex-col">
                <span class="font-bold text-lg mb-1">{lookup.name()}</span>
                <span class="text-neutral-400">{lookup.display_name()}</span>
            </div>
        </div>
    }
}
