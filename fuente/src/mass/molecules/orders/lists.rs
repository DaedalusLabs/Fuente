use yew::prelude::*;

use crate::models::OrderStatus;

#[derive(Clone, PartialEq, Properties)]
pub struct OrderListProps {
    pub title: OrderStatus,
    pub children: Children,
}

#[function_component(OrderList)]
pub fn order_list(props: &OrderListProps) -> Html {
    let column_id = props.title.to_string();
    let button_class = classes!(
        "text-sm",
        "font-bold",
        "px-2",
        "py-3",
        "border-2",
        props.title.border_color(),
        "rounded-lg"
    );
    let button_text_class = classes!(
        "text-lg",
        "font-semibold",
        "text-center",
        "text-nowrap",
        props.title.text_color()
    );
    let column_class = classes!(
        "h-[500px]",
        "overflow-y-scroll",
        "mt-2",
        "rounded-2xl",
        "px-2",
        "py-2",
        "space-y-3",
        "no-scrollbar",
        props.title.theme_color()
    );

    html! {
        <section>
            <div class={button_class}>
                <p class={button_text_class}>
                    {&props.title.display()}
                </p>
            </div>
            <div
                id={column_id}
                class={column_class}>
                {props.children.clone()}
            </div>
        </section>
    }
}
