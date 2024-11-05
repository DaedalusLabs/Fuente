use std::rc::Rc;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NostrId {
    has_loaded: bool,
    identity: Option<super::nostr_id::UserIdentity>,
    keys: Option<nostro2::userkeys::UserKeys>,
}
impl NostrId {
    pub fn finished_loading(&self) -> bool {
        self.has_loaded
    }
    pub fn get_nostr_key(&self) -> Option<nostro2::userkeys::UserKeys> {
        self.keys.clone()
    }
    pub fn get_identity(&self) -> Option<super::nostr_id::UserIdentity> {
        self.identity.clone()
    }
}

pub enum NostrIdAction {
    FinishedLoadingKey,
    LoadIdentity(super::nostr_id::UserIdentity, nostro2::userkeys::UserKeys),
}
impl Reducible for NostrId {
    type Action = NostrIdAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            NostrIdAction::LoadIdentity(identity, key) => Rc::new(NostrId {
                has_loaded: self.has_loaded,
                identity: Some(identity),
                keys: Some(key),
            }),
            NostrIdAction::FinishedLoadingKey => Rc::new(NostrId {
                has_loaded: true,
                identity: self.identity.clone(),
                keys: self.keys.clone(),
            }),
        }
    }
}
pub type NostrIdStore = UseReducerHandle<NostrId>;

#[function_component(NostrIdProvider)]
pub fn key_handler(props: &yew::html::ChildrenProps) -> Html {
    let ctx = use_reducer(|| NostrId {
        has_loaded: false,
        identity: None,
        keys: None,
    });

    let ctx_clone = ctx.clone();
    use_effect_with((), |_| {
        spawn_local(async move {
            if let Ok(id) = super::nostr_id::UserIdentity::find_local_identity().await {
                let keys = id.get_user_keys().await.expect("Error getting user keys");
                ctx_clone.dispatch(NostrIdAction::LoadIdentity(id, keys));
                ctx_clone.dispatch(NostrIdAction::FinishedLoadingKey);
            } else {
                ctx_clone.dispatch(NostrIdAction::FinishedLoadingKey);
                gloo::console::error!("Loaded with no keys");
            }
        });
        || {}
    });

    html! {
        <ContextProvider<NostrIdStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<NostrIdStore>>
    }
}
