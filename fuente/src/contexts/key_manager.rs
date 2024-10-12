use std::rc::Rc;

use crate::models::consumer_id::UserIdentity;
use nostro2::userkeys::UserKeys;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NostrId {
    has_loaded: bool,
    identity: Option<UserIdentity>,
    keys: Option<UserKeys>,
}

impl NostrId {
    pub fn finished_loading(&self) -> bool {
        self.has_loaded
    }
    pub fn get_key(&self) -> Option<UserKeys> {
        self.keys.clone()
    }
}

pub enum NostrIdAction {
    FinishedLoadingKey,
    LoadIdentity(UserIdentity, UserKeys),
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

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct NostrIdChildren {
    pub children: Children,
}

#[function_component(NostrIdProvider)]
pub fn key_handler(props: &NostrIdChildren) -> Html {
    let ctx = use_reducer(|| NostrId {
        has_loaded: false,
        identity: None,
        keys: None,
    });

    let ctx_clone = ctx.clone();
    use_effect_with((), |_| {
        spawn_local(async move {
            if let Ok(id) = UserIdentity::find_local_identity().await {
                let keys = id.get_user_keys().await.unwrap();
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
