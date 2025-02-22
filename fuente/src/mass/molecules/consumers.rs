use yew::prelude::*;

use crate::{
    contexts::LanguageConfigsStore,
    models::{ConsumerAddress, ConsumerProfile},
};

#[derive(Clone, PartialEq, Properties)]
pub struct CustomerDetailsProps {
    pub customer: ConsumerProfile,
}
#[function_component(CustomerDetails)]
pub fn customer_details(props: &CustomerDetailsProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let CustomerDetailsProps { customer } = props;
    html! {
        <section class="space-y-3 border-t border-t-gray-400 md:border-t-0 py-3 w-full mt-5">
            <h3 class="text-gray-500 font-light text-lg">{&translations["store_order_modal_customer"]}</h3>
            <p class="text-gray-500 font-bold text-lg">{&customer.nickname}</p>
            <div class="w-full flex flex-col gap-2 justify-between">
                <div class="flex gap-2 items-center">
                    <p class="text-gray-500 font-bold text-lg">{&translations["checkout_client_information_heading_email"]}</p>
                    <p class="text-gray-500 flex-1 text-right">{&customer.email}</p>
                </div>

                <div class="flex gap-2 items-center">
                    <p class="text-gray-500 font-bold text-lg">{&translations["checkout_client_information_heading_phone"]}</p>
                    <p class="text-gray-500 flex-1 text-right">{&customer.telephone}</p>
                </div>
            </div>
        </section>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct CustomerAddressDetailsProps {
    pub customer: ConsumerAddress,
}
#[function_component(CustomerAddressDetails)]
pub fn customer_details(props: &CustomerAddressDetailsProps) -> Html {
    let CustomerAddressDetailsProps { customer } = props;
    html! {
        <section class="space-y-3 border-b border-b-gray-400 py-3 w-full text-wrap">
            <p class="text-gray-500 line-clamp-3">{&customer.lookup().display_name()}</p>
        </section>
    }
}
