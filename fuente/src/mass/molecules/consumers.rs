use yew::prelude::*;

use crate::models::ConsumerProfile;

#[derive(Properties, Clone, PartialEq)]
pub struct ConsumerProfileProps {
    pub consumer_profile: ConsumerProfile,
}

#[function_component(ConsumerProfileDetails)]
pub fn consumer_profile_card(props: &ConsumerProfileProps) -> Html {
    let ConsumerProfileProps { consumer_profile } = props;
    html! {
        <div class="flex flex-row gap-4">
            <div class="w-16 h-16 bg-neutral-200 rounded-2xl"></div>
            <div class="flex flex-col">
                <span class="font-bold text-lg mb-1">{&consumer_profile.nickname}</span>
                <span class="text-neutral-400">{&consumer_profile.telephone}</span>
                <span class="text-neutral-400">{&consumer_profile.email}</span>
            </div>
        </div>
    }
}
