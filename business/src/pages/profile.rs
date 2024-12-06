use crate::contexts::{CommerceDataAction, CommerceDataStore};

use fuente::{
    mass::{
        BackArrowIcon, CardComponent, CommerceProfileAddressDetails, CommerceProfileDetails,
        CommerceProfileProps, DrawerSection, NewAddressForm, NewAddressProps, SimpleInput,
        SimpleTextArea,
    },
    models::{CommerceProfile, CommerceProfileIdb},
};
use minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::{prelude::*, props};

#[derive(Clone, PartialEq)]
pub enum ProfilePageMenu {
    None,
    EditContactDetails,
    EditBusinessAddress,
}

#[function_component(ProfilePage)]
pub fn profile_page() -> Html {
    let menu_state = use_state(|| ProfilePageMenu::None);
    html! {
        <div class="flex flex-col flex-1 gap-8">
            {match &(*menu_state) {
                ProfilePageMenu::None => html! {<>
                    <h2 class="text-4xl">{"My Business Profile"}</h2>
                    <MyContactDetails handle={menu_state.clone()} />
                </>},
                ProfilePageMenu::EditContactDetails => html! {<>
                    <MenuHeader title={"My Profile".to_string()} handle={menu_state.clone()} />
                    <EditProfileMenu handle={menu_state.clone()} />
                </>},
                ProfilePageMenu::EditBusinessAddress => html! {<>
                    <MenuHeader title={"My Profile".to_string()} handle={menu_state.clone()} />
                    <EditProfileMenu handle={menu_state.clone()} />
                </>},
            }}
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct MenuHeaderProps {
    pub title: String,
    pub handle: UseStateHandle<ProfilePageMenu>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct MenuProps {
    pub handle: UseStateHandle<ProfilePageMenu>,
}

#[function_component(MenuHeader)]
pub fn page_header(props: &MenuHeaderProps) -> Html {
    let handle = props.handle.clone();
    html! {
        <div class="w-full flex flex-row justify-between">
            <button class="" onclick={Callback::from(move |_|{ handle.set(ProfilePageMenu::None)})}>
                <BackArrowIcon class="w-8 h-8 stroke-black" />
            </button>
            <h3 class="flex-1 text-center text-lg font-semibold">{&props.title}</h3>
            <div class="h-8 w-8"></div>
        </div>
    }
}

#[function_component(MyContactDetails)]
pub fn my_contact_details(props: &MenuProps) -> Html {
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let profile = user_ctx.profile().expect("No user profile found");

    let contact_handle = props.handle.clone();
    let contact_cb =
        Callback::from(move |_| contact_handle.set(ProfilePageMenu::EditContactDetails));

    let business_handle = props.handle.clone();
    let business_cb =
        Callback::from(move |_| business_handle.set(ProfilePageMenu::EditBusinessAddress));

    html! {
        <div class="flex flex-col flex-1 gap-8 max-w-xl">
            <div class="flex flex-col gap-2">
                <div class="w-full flex flex-row justify-between items-center pr-4">
                    <h3 class="font-bold tracking-wide">{"Contact Details"}</h3>
                    <button
                        onclick={contact_cb}
                        class="font-semibold tracking-wide underline underline-offset-2 text-fuente">
                            {"Edit"}
                        </button>
                </div>
                <CardComponent>
                    <CommerceProfileDetails commerce_data={profile.clone()} />
                </CardComponent>
            </div>
            <div class="flex flex-col gap-2">
                <div class="w-full flex flex-row justify-between items-center pr-4">
                    <h3 class="font-bold tracking-wide">{"Business Address"}</h3>
                    <button
                        onclick={business_cb}
                        class="font-semibold tracking-wide underline underline-offset-2 text-fuente">
                            {"Edit"}
                        </button>
                </div>
                <CardComponent>
                    <CommerceProfileAddressDetails commerce_data={profile.clone()} />
                </CardComponent>
            </div>
        </div>
    }
}

#[function_component(EditProfileMenu)]
pub fn edit_profile_menu(props: &MenuProps) -> Html {
    let MenuProps { handle } = props;
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let profile = user_ctx.profile().expect("No user profile found");
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();
    let handle = handle.clone();

    let coordinate_state = use_state(|| None);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
    let props = props!(NewAddressProps {
        coord_handle: coordinate_state.clone(),
        nominatim_handle: nominatim_state.clone(),
        map_handle: map_state,
        marker_handle: marker_state,
        onclick: Callback::from(move |_: MouseEvent| {}),
    });

    let coords = (*coordinate_state).clone();
    let address = (*nominatim_state).clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let user_keys = keys.clone();
        let handle = handle.clone();
        let sender = sender.clone();
        let user_ctx = user_ctx.clone();
        let new_profile = CommerceProfile::new(
            form.input_value("name").expect("Failed to get name"),
            form.textarea_value("description")
                .expect("Failed to get description"),
            form.input_value("telephone")
                .expect("Failed to get telephone"),
            form.input_value("web").expect("Failed to get web"),
            address.clone().expect("No address found"),
            coords.clone().expect("No coordinates found"),
            form.input_value("ln_address")
                .expect("Failed to get lightning address"),
        );
        let db = CommerceProfileIdb::new(new_profile.clone(), &user_keys)
            .expect("Failed to create profile");
        let note = db.signed_note();
        sender.emit(note.clone());
        user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
        handle.set(ProfilePageMenu::None);
    });
    let details_card_state = use_state(|| false);
    let address_card_state = use_state(|| false);
    html! {
        <form {onsubmit}
            class="w-full h-full flex flex-col gap-4 overflow-y-scroll p-8">
            <div class="flex flex-row w-full justify-between items-center pr-4">
                <h3 class="font-bold">{"Edit Profile"}</h3>
                <button
                    type="submit"
                    class="text-sm bg-fuente text-white font-bold p-2 px-4 rounded-3xl"
                    >{"Save"}</button>
            </div>
            <DrawerSection title={"Edit Details"} open={details_card_state.clone()}>
                <NewAddressInputs commerce_data={profile.clone()} />
            </DrawerSection>
            <DrawerSection title={"Edit Address"} open={address_card_state.clone()}>
                <NewAddressForm ..props />
            </DrawerSection>
        </form>
    }
}

#[function_component(NewAddressInputs)]
pub fn new_address_inputs(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
        <div class="flex flex-col px-4 gap-2">
            <SimpleInput
                id="name"
                name="name"
                label="Name"
                value={commerce_data.name().to_string()}
                input_type="text"
                required={true}
            />
            <SimpleInput
                id="telephone"
                name="telephone"
                label="Telephone"
                value={commerce_data.telephone().to_string()}
                input_type="tel"
                required={true}
            />
            <SimpleInput
                id="web"
                name="web"
                label="Web"
                value={commerce_data.web().to_string()}
                input_type="text"
                required={true}
            />
            <SimpleInput
                id="ln_address"
                name="ln_address"
                label="Lightning Address"
                value={commerce_data.ln_address().to_string()}
                input_type="text"
                required={true}
            />
            <SimpleTextArea
                id="description"
                name="description"
                label="Description"
                value={commerce_data.description().to_string()}
                input_type="text"
                required={true}
            />
        </div>
    }
}
