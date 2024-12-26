use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppLocale {
    English,
    Dutch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageConfigs {
    locale: AppLocale,
    translations: TranslationData,
}
impl LanguageConfigs {
    pub fn translations(&self) -> &std::collections::HashMap<String, String> {
        &self.translations.translations
    }
}

pub enum LanguageConfigsAction {
    ChangeLocale(AppLocale),
}
impl Reducible for LanguageConfigs {
    type Action = LanguageConfigsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            LanguageConfigsAction::ChangeLocale(locale) => Rc::new(LanguageConfigs {
                locale,
                translations: TranslationData::load_translation(locale),
            }),
        }
    }
}
pub type LanguageConfigsStore = UseReducerHandle<LanguageConfigs>;

#[function_component(LanguageConfigsProvider)]
pub fn key_handler(props: &yew::html::ChildrenProps) -> Html {
    let ctx = use_reducer(|| LanguageConfigs {
        locale: AppLocale::English,
        translations: TranslationData::default(),
    });

    html! {
        <ContextProvider<LanguageConfigsStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<LanguageConfigsStore>>
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TranslationData {
    #[serde(flatten)]
    pub translations: HashMap<String, String>,
}
impl Default for TranslationData {
    fn default() -> Self {
        Self::load_translation(AppLocale::English)
    }
}
impl TranslationData {
    pub fn load_translation(locale: AppLocale) -> Self {
        match locale {
            AppLocale::English => serde_json::from_str(ENGLISH_TRANSLATIONS).unwrap(),
            AppLocale::Dutch => serde_json::from_str(DUTCH_TRANSLATIONS).unwrap(),
        }
    }
}

static ENGLISH_TRANSLATIONS: &str = include_str!("../../../../public/language/en.json");
static DUTCH_TRANSLATIONS: &str = include_str!("../../../../public/language/nl.json");
