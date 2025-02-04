use lucide_yew::X;
use yew::prelude::*;
use nostr_minions::key_manager::NostrIdStore;
use yew_router::hooks::use_navigator;
use fuente::mass::LoginPage;
use crate::router::ConsumerRoute;

#[derive(Properties, Clone, PartialEq)]
pub struct RequireAuthProps {
    pub children: Children,
}

// Create a state to manage login modal visibility globally
#[derive(Default, PartialEq, Clone)]
pub struct LoginState {
    pub show_modal: bool
}

pub enum LoginStateAction {
    Show,
    Hide
}

impl Reducible for LoginState {
    type Action = LoginStateAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            LoginStateAction::Show => std::rc::Rc::new(Self { show_modal: true }),
            LoginStateAction::Hide => std::rc::Rc::new(Self { show_modal: false })
        }
    }
}

pub type LoginStateStore = UseReducerHandle<LoginState>;

#[derive(Properties, Clone, PartialEq)]
pub struct LoginProviderProps {
    pub children: Children,
}

#[function_component(LoginProvider)]
pub fn login_provider(props: &LoginProviderProps) -> Html {
    let state = use_reducer(LoginState::default);

    html! {
        <ContextProvider<LoginStateStore> context={state}>
            {props.children.clone()}
        </ContextProvider<LoginStateStore>>
    }
}

#[function_component(RequireAuth)]
pub fn require_auth(props: &RequireAuthProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let login_state = use_context::<LoginStateStore>().expect("LoginStateStore not found");

    let is_authenticated = key_ctx.get_nostr_key().is_some();

    if !is_authenticated {
        login_state.dispatch(LoginStateAction::Show);
        html! {
            <div class="h-screen flex items-center justify-center">
                <p>{"Please log in to access this feature"}</p>
            </div>
        }
    } else {
        html! { 
            {props.children.clone()} 
        }
    }
}

#[function_component(LoginModal)]
pub fn login_modal() -> Html {
    let login_state = use_context::<LoginStateStore>().expect("LoginStateStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let navigator = use_navigator().expect("Navigator not found");
    
    // Watch for auth state changes
    {
        let login_state = login_state.clone();
        use_effect_with(key_ctx.clone(), move |key_ctx| {
            if key_ctx.get_nostr_key().is_some() {
                // User is logged in, hide modal
                login_state.dispatch(LoginStateAction::Hide);
            }
            || {}
        });
    }
    
    if !login_state.show_modal {
        return html! {};
    }

    html! {
        <div class="fixed inset-0 z-50 overflow-hidden">
            <div class="fixed inset-0 bg-black opacity-40"></div>
            <div class="fixed inset-0 flex items-center justify-center p-4">
                <div class="relative bg-white rounded-lg shadow-xl w-full max-w-xl mx-auto">
                    <button 
                        onclick={
                            let nav = navigator.clone();
                            let login_state = login_state.clone();
                            Callback::from(move |_| {
                                login_state.dispatch(LoginStateAction::Hide);
                                nav.push(&ConsumerRoute::Home);
                            })
                        }
                        class="absolute right-4 top-4 text-gray-500 hover:text-gray-700 z-50"
                    >
                        <X class="w-6 h-6" />
                    </button>
                    <div class="p-8 w-full overflow-y-auto max-h-[90vh]">
                        <LoginPage />
                    </div>
                </div>
            </div>
        </div>
    }
}