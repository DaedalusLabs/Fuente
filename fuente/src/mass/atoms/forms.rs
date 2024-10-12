use html::ChildrenProps;
use yew::prelude::*;
use yew_router::{
    hooks::{use_navigator, use_route},
    Routable,
};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SimpleInputProps {
    pub id: String,
    pub name: String,
    pub input_type: String,
    pub value: String,
    pub required: bool,
    pub label: String,
}

#[function_component(SimpleInput)]
pub fn simple_input(props: &SimpleInputProps) -> Html {
    let value = props.value.clone();
    let id = props.id.clone();
    let name = props.name.clone();
    let input_type = props.input_type.clone();
    let required = props.required;
    let label = props.label.clone();
    html! {
        <div class="w-full">
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <input
                {id}
                {name}
                type={input_type}
                {value}
                {required}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
                />
        </div>
    }
}

#[function_component(SimpleTextArea)]
pub fn simple_textarea(props: &SimpleInputProps) -> Html {
    let value = props.value.clone();
    let id = props.id.clone();
    let name = props.name.clone();
    let required = props.required;
    let label = props.label.clone();
    html! {
        <div class="w-full">
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <textarea
                {id}
                {name}
                {value}
                {required}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
                />
        </div>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct SimpleSelectProps {
    pub id: String,
    pub name: String,
    pub label: String,
    pub children: Children,
}

#[function_component(SimpleSelect)]
pub fn simple_select(props: &SimpleSelectProps) -> Html {
    let id = props.id.clone();
    let name = props.name.clone();
    let required = true;
    let label = props.label.clone();
    html! {
        <div class="w-full">
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <select
                {id}
                {name}
                {required}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
                >
                {props.children.clone()}
            </select>
        </div>
    }
}

#[function_component(SimpleFormButton)]
pub fn simple_button(props: &ChildrenProps) -> Html {
    html! {
        <button type={"submit"}
            class="bg-fuente-light text-white font-mplus p-4 mx-16 rounded-3xl
            focus:outline-none focus:shadow-outline hover:bg-fuente-dark m-8"
            >
            {props.children.clone()}
        </button>
    }
}

#[function_component(MoneyInput)]
pub fn money_input(props: &SimpleInputProps) -> Html {
    let value = props.value.clone();
    let id = props.id.clone();
    let name = props.name.clone();
    let required = props.required;
    let label = props.label.clone();
    html! {
        <div class="w-full">
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <input
                {id}
                {name}
                type={"number"}
                {value}
                {required}
                step={"0.01"}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
                />
        </div>
    }
}


#[derive(Clone, Debug, Properties, PartialEq)]
pub struct AppLinkProps<T>
where
    T: Routable,
{
    pub children: Children,
    pub class: String,
    pub selected_class: String,
    pub route: T,
}

#[function_component(AppLink)]
pub fn sidebar_link<T>(props: &AppLinkProps<T>) -> Html
where
    T: Routable + 'static,
{
    let navigator = use_navigator();
    if navigator.is_none() {
        return html! {};
    }
    let navigator = navigator.unwrap();
    let current_route = use_route::<T>().unwrap();

    let onclick = {
        let route = props.route.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| navigator.push(&route))
    };
    let class = if current_route == props.route {
        props.selected_class.clone()
    } else {
        props.class.clone()
    };
    html! {
        <button {onclick} {class}>
            {props.children.clone()}
        </button>
    }
}
