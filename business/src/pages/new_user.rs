use fuente::{
    mass::{ImageUploadInput, NewAddressForm, NewAddressProps, SimpleInput, SimpleTextArea},
    models::{CommerceProfile, CommerceProfileIdb},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::{prelude::*, props};

use crate::contexts::{CommerceDataAction, CommerceDataStore};

#[function_component(NewProfilePage)]
pub fn edit_profile_menu() -> Html {
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let logo_url = use_state(|| None);
    let logo_handle = logo_url.clone();

    let banner_url = use_state(|| None);
    let banner_handle = banner_url.clone();

    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();

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
            logo_url.as_ref().cloned().expect("No profile pic found"),
            banner_url.as_ref().cloned().expect("No banner found"),
        );
        let db = CommerceProfileIdb::new(new_profile.clone(), &user_keys)
            .expect("Failed to create profile");
        let note = db.signed_note();
        sender.emit(note.clone());
        user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
    });
    let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
    html! {
        <form {onsubmit}
            class="w-full h-full flex flex-col gap-4 px-8 py-4">
                <div class="flex flex-row w-full justify-between items-center pr-4">
                    <h3 class="font-bold">{"Business Details"}</h3>
                    <button
                        type="submit"
                        class="text-sm bg-purple-900 text-white font-bold p-2 px-4 rounded-3xl"
                        >{"Save"}</button>
                </div>
                <ProfileInputs />
                <ImageUploadInput
                    url_handle={logo_handle} nostr_keys={nostr_keys.clone()}
                    classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")}
                    input_id="logo-image-upload"/>
                <ImageUploadInput
                    url_handle={banner_handle} nostr_keys={nostr_keys}
                    classes={classes!("min-w-64", "min-h-32", "h-32", "w-64")}
                    input_id="banner-image-upload"/>
            <NewAddressForm ..props />
        </form>
    }
}

#[function_component(ProfileInputs)]
fn profile_inputs() -> Html {
    html! {
        <div class="flex flex-col gap-2">
            <SimpleInput
                id="name"
                name="name"
                label="Name"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleInput
                id="telephone"
                name="telephone"
                label="Telephone"
                value=""
                input_type="tel"
                required={true}
                />
            <SimpleInput
                id="web"
                name="web"
                label="Website"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleInput
                id="ln_address"
                name="ln_address"
                label="Lightning Address"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleTextArea
                id="description"
                name="description"
                label="Description"
                value=""
                input_type="text"
                required={true}
                />
        </div>
    }
}
