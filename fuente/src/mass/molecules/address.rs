use crate::widgets::leaflet::NominatimLookup;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct LookupProps {
    pub lookup: NominatimLookup,
}

#[function_component(AddressLookupDetails)]
pub fn consumer_address_card(props: &LookupProps) -> Html {
    let LookupProps { lookup } = props;
    let mut split = lookup.display_name().split(",");
    let name = split.next().unwrap().to_string();
    let display_name = split.collect::<Vec<&str>>().join(",");
    html! {
        <div class="flex flex-row gap-4">
            <div class="min-w-16 min-h-16 h-16 w-16 bg-neutral-200 rounded-2xl"></div>
            <div class="flex flex-col gap-1 text-wrap shrink">
                <span class="text-sm font-bold">{&name}</span>
                <span class="text-xs text-neutral-400 overflow-hidden text-ellipsis">
                    {&display_name}
                </span>
            </div>
        </div>
    }
}
