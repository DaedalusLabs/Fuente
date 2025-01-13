use fuente::contexts::LanguageConfigsStore;
use fuente::mass::templates::{KeyRecoverySection, SettingsPageTemplate};
use fuente::mass::LanguageToggle;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum SettingsPage {
    KeyRecovery,
    Language,
}

#[function_component(SettingsPageComponent)]
pub fn settings_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();
    let current_page = use_state(|| SettingsPage::Language);
    let go_to_key_recovery = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(SettingsPage::KeyRecovery))
    };
    let go_to_language = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(SettingsPage::Language))
    };
    let edit_button = {
        match *current_page {
            _ => {
                html! {}
            }
        }
    };
    html! {
        <SettingsPageTemplate
            heading={translations["admin_settings_title_settings"].clone()}
            options={ vec![]}
            sidebar_options={ vec![
                (translations["profile_settings_key"].clone(), go_to_key_recovery, if *current_page == SettingsPage::KeyRecovery { true } else { false }),
                (translations["profile_settings_language"].clone(), go_to_language, if *current_page == SettingsPage::Language { true } else { false }),
            ]}
            content_button={Some(edit_button)} >
            <>
            {match *current_page {
                    SettingsPage::KeyRecovery => html! {
                        <div class="w-full">
                            <KeyRecoverySection />
                        </div>
                    },
                    SettingsPage::Language => html! {
                        <div class="w-full">
                            <LanguageToggle />
                        </div>
                    },
            }}
            </>
        </SettingsPageTemplate>
    }
}

