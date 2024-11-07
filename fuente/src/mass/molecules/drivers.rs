use yew::prelude::*;

use crate::models::driver::DriverProfile;

#[derive(Clone, PartialEq, Properties)]
pub struct DriverDetailsProps {
    pub pubkey: String,
    pub driver: DriverProfile,
}

#[function_component(DriverDetailsComponent)]
pub fn driver_details(props: &DriverDetailsProps) -> Html {
    html! {
        <div class="flex flex-row items-center w-fit h-fit gap-2 p-2">
            <div class="w-16 h-16 min-w-16 min-h-16 max-h-16 max-w-16 rounded-sm bg-gray-300">
            </div>
            <div class="flex flex-col gap-1">
                <p>{"ID: "}{&props.pubkey[..12].to_string()}</p>
                <p>{props.driver.nickname()}</p>
                <p>{props.driver.telephone()}</p>
            </div>
        </div>
    }
}
