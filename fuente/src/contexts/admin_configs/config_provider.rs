use std::rc::Rc;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminConfigs {
    has_loaded: bool,
}
impl AdminConfigs {}

pub enum AdminConfigsAction {}
impl Reducible for AdminConfigs {
    type Action = AdminConfigsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {}
    }
}
pub type AdminConfigsStore = UseReducerHandle<AdminConfigs>;

#[function_component(AdminConfigsProvider)]
pub fn key_handler(props: &yew::html::ChildrenProps) -> Html {
    let ctx = use_reducer(|| AdminConfigs {
        has_loaded: false,
    });

    let ctx_clone = ctx.clone();
    use_effect_with((), |_| {
        spawn_local(async move {
        });
        || {}
    });

    html! {
        <ContextProvider<AdminConfigsStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<AdminConfigsStore>>
    }
}
