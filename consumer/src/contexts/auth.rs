use yew::prelude::*;
use fuente::mass::LoginPage;
use nostr_minions::key_manager::NostrIdStore;
use yew_router::hooks::use_navigator;

use crate::router::ConsumerRoute;

#[derive(Properties, Clone, PartialEq)]
pub struct ProtectedRouteProps {
    pub children: Children,
    pub fallback: Html,
}

#[function_component(ProtectedRoute)]
pub fn protected_route(props: &ProtectedRouteProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let navigator = use_navigator().unwrap();  // Moved hook to top-level
    
    if !key_ctx.finished_loading() {
        return html! {};
    }

    if key_ctx.get_nostr_key().is_none() {
        navigator.push(&ConsumerRoute::Home);
        return html! {};
    }

    html! {
        {props.children.clone()}
    }
}

#[function_component(LoginPrompt)]
pub fn login_prompt() -> Html {
    let navigator = use_navigator().unwrap();

    // Moved hook to top-level
    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white p-8 rounded-lg w-full max-w-md">
                <LoginPage />
            </div>
        </div>
    }
}