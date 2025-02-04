use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Success,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastState {
    toasts: Vec<Toast>,
}

pub enum ToastAction {
    Show(Toast),
    Hide,
}

impl Reducible for ToastState {
    type Action = ToastAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ToastAction::Show(toast) => {
                let mut toasts = self.toasts.clone();
                toasts.push(toast);
                Rc::new(ToastState { toasts })
            }
            ToastAction::Hide => {
                let mut toasts = self.toasts.clone();
                if !toasts.is_empty() {
                    toasts.remove(0);
                }
                Rc::new(ToastState { toasts })
            }
        }
    }
}

pub type ToastContext = UseReducerHandle<ToastState>;

#[derive(Properties, Clone, PartialEq)]
pub struct ToastProviderProps {
    pub children: Children,
}

#[function_component(ToastProvider)]
pub fn toast_provider(props: &ToastProviderProps) -> Html {
    let toast_state = use_reducer(|| ToastState { toasts: vec![] });

    html! {
        <ContextProvider<ToastContext> context={toast_state.clone()}>
            {props.children.clone()}
            <ToastContainer toasts={toast_state.toasts.clone()} />
        </ContextProvider<ToastContext>>
    }
}

#[derive(Properties, Clone, PartialEq)]
struct ToastContainerProps {
    toasts: Vec<Toast>,
}

#[function_component(ToastContainer)]
fn toast_container(props: &ToastContainerProps) -> Html {
    let toast_ctx = use_context::<ToastContext>().expect("No toast context found");

    {
        let toast_ctx = toast_ctx.clone();
        use_effect_with(props.toasts.clone(), move |toasts| {
            if !toasts.is_empty() {
                let handle = gloo::timers::callback::Timeout::new(3000, move || {
                    toast_ctx.dispatch(ToastAction::Hide);
                });
                handle.forget();
            }
            || {}
        });
    }

    html! {
        <div class="fixed top-4 right-4 z-50 flex flex-col gap-2">
            {props.toasts.iter().map(|toast| {
                let bg_color = match toast.toast_type {
                    ToastType::Success => "bg-green-500",
                    ToastType::Error => "bg-red-500",
                };
                html! {
                    <div class={classes!("px-4", "py-2", "rounded-lg", "text-white", "shadow-lg", bg_color)}>
                        {&toast.message}
                    </div>
                }
            }).collect::<Html>()}
        </div>
    }
}