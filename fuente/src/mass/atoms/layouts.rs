use html::ChildrenProps;
use lucide_yew::ChevronRight;
use yew::prelude::*;

#[function_component(MainLayout)]
pub fn layout(props: &ChildrenProps) -> Html {
    html! {
        <div class="h-dvh w-dvw flex flex-col md:flex-row-reverse">
            {props.children.clone()}
        </div>
    }
}

#[function_component(MainPanel)]
pub fn layout(props: &ChildrenProps) -> Html {
    html! {
        <div class="flex flex-1 flex-col pt-8">
            {props.children.clone()}
        </div>
    }
}

#[function_component(LoadingScreen)]
pub fn loading_screen() -> Html {
    html! {
        <div class="flex justify-center items-center flex-1 h-full w-full py-8 px-16">
            <img src="/public/assets/img/logo.jpg" />
        </div>
    }
}

#[function_component(CardComponent)]
pub fn card_component(props: &ChildrenProps) -> Html {
    html! {
        <div class="bg-neutral-50 shadow-xl p-4 rounded-3xl">
            {props.children.clone()}
        </div>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct DrawerProps {
    pub children: Children,
    pub title: String,
    pub open: UseStateHandle<bool>,
}

#[function_component(DrawerSection)]
pub fn drawer_section(props: &DrawerProps) -> Html {
    let DrawerProps {
        open,
        title,
        children,
    } = props;
    let onclick = {
        let close_handle = open.clone();
        Callback::from(move |_| close_handle.set(!*close_handle))
    };
    let icon_class = if !**open {
        "w-8 h-8 stroke-purple-900"
    } else {
        "transform rotate-90 w-8 h-8 stroke-purple-900"
    };
    let children_class = if !**open {
        "hidden"
    } else {
        "flex flex-col gap-4"
    };
    html! {
        <>
        <button
            type="button"
            {onclick}>
            <CardComponent>
                <div class="flex flex-row w-full gap-4 items-center">
                    <div class="w-16 h-16 min-w-16 min-h-16 bg-neutral-300 rounded-xl"></div>
                    <h3 class="text-2xl font-bold flex-1 flex">{title}</h3>
                        <ChevronRight class={icon_class} />
                </div>
            </CardComponent>
        </button>
        <div class={children_class}>
            {children.clone()}
        </div>
        </>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct PopupProps {
    #[prop_or_default]
    pub children: Children,
    pub close_handle: UseStateHandle<bool>,
}

#[function_component(PopupSection)]
pub fn popup_section(props: &PopupProps) -> Html {
    if !*props.close_handle {
        return html! {};
    };
    let onclick = {
        let close_handle = props.close_handle.clone();
        Callback::from(move |_| close_handle.set(false))
    };
    html! {
        <>
        <div {onclick}
            class="fixed inset-0 bg-neutral-900 opacity-40 z-[500]">
        </div>
        <div class="absolute inset-0 flex items-center justify-center p-8 z-[501] pointer-events-none">
            <div class="h-fit w-fit flex-col pointer-events-auto">
                {props.children.clone()}
            </div>
        </div>
        </>
    }
}
