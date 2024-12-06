use super::PageHeader;
use fuente::mass::{HistoryIcon, SimpleFormButton};
use yew::prelude::*;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    html! {
        <div class="h-full w-full flex flex-col justify-between items-center">
            <PageHeader title={"History".to_string()} />
            <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                <HistoryIcon class="w-32 h-32 stroke-neutral-200" />
                <h4 class="text-xl font-semibold mt-4">{"No history yet"}</h4>
                <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48  text-center text-wrap">
                    {"Hit the button below to create a new order!"}
                </p>
            </div>
            <SimpleFormButton >
                {"Create an Order"}
            </SimpleFormButton>
        </div>
    }
}
