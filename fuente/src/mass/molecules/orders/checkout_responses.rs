use lucide_yew::{Bitcoin, Frown};
use yew::prelude::*;

use crate::{contexts::LanguageConfigsStore, models::{OrderInvoiceState, OrderStatus}};
#[derive(Clone, PartialEq, Properties)]
pub struct OrderConfirmationProps {
    pub order: OrderInvoiceState,
    pub onclick: Callback<MouseEvent>,
}

#[function_component(CheckoutBannerTemplate)]
pub fn settings_template(props: &OrderConfirmationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let order_state = &props.order.order_status;
    let number_class = classes!(
        "text-white",
        "w-10",
        "h-10",
        "lg:w-16",
        "lg:h-16",
        "rounded-full",
        "font-bold",
        "text-lg",
        "text-center",
        "flex",
        "items-center",
        "justify-center"
    );
    let text_class = classes!("font-bold", "text-md", "md:text-lg", "lg:text-xl");
    let (confirmation_color, confirmation_text) = if order_state == &OrderStatus::Pending {
        ("bg-orange-500", "text-orange-500")
    } else {
        ("bg-fuente", "text-fuente")
    };
    let (payment_color, payment_text) = if order_state != &OrderStatus::Pending {
        ("bg-orange-500", "text-orange-500")
    } else {
        ("bg-fuente", "text-fuente")
    };
    html! {
        <>
        <div class="bg-gray-100 py-5 flex justify-center items-center gap-5 lg:gap-16">
            <div class="flex items-center gap-5">
                <p class={classes!(number_class.clone(), confirmation_color)}>{"1"}</p>
                <p class={classes!(text_class.clone(), confirmation_text)}>{&translations["payment_step_1"]}</p>
            </div>

            <div class="flex items-center justify-end gap-4">
                <p class={classes!(number_class, payment_color)}>{"2"}</p>
                <p class={classes!(text_class, payment_text)}>{&translations["payment_step_2"]}</p>
            </div>
        </div>
        </>
    }
}
#[function_component(OrderPendingTemplate)]
pub fn settings_template() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    html! {
        <>
            <main class="mt-20">
                <div class="w-52 mx-auto flex justify-center items-center">
                    <Bitcoin class="text-fuente-orange" size=128 />
                </div>

                <div class="max-w-md mx-auto mt-5 space-y-3">
                    <h1 class="text-3xl font-bold text-fuente text-center tracking-tighter">
                        {&translations["payment_heading"]}
                    </h1>
                    <p class="font-light text-fuente text-center w-5/6 mx-auto">
                        {&translations["payment_detail"]}
                    </p>
                </div>
            </main>
        </>
    }
}
#[function_component(OrderSuccessTemplate)]
pub fn settings_template(props: &OrderConfirmationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let order = &props.order;
    let onclick = props.onclick.clone();
    html! {
        <>
        <main class="mt-20">
            <div class="w-52 mx-auto flex justify-center items-center">
                <Bitcoin class="text-fuente-orange" size=128 />
            </div>

            <div class="flex flex-col items-center mt-6 space-y-10">
                <div>
                    <h1 class="text-3xl text-fuente text-center font-bold">{&translations["confirmation_heading"]}</h1>
                    <p class="text-3xl text-fuente text-center font-bold">{&translations["confirmation_text"]}</p>
                </div>

                <h3 class="text-fuente-orange font-bold text-2xl">{format!("#{}", &order.order_id()[..8])}</h3>

                <div class="flex flex-col items-center justify-center">
                    <p class="font-light text-center text-fuente">{&translations["confirmation_detail"]}</p>
                    <p class="font-light text-center text-fuente">{&translations["confirmation_detail_message"]}</p>
                    <button {onclick} class="bg-fuente-buttons text-fuente-forms py-4 px-7 rounded-full mt-5 font-bold">
                        {&translations["confirmation_button"]}
                    </button>
                </div>
            </div>
        </main>
        </>
    }
}
#[function_component(OrderFailureTemplate)]
pub fn settings_template(props: &OrderConfirmationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let order = &props.order;
    html! {
        <>
            <main class="mt-20">
                <div class="w-52 mx-auto flex justify-center items-center">
                    <Frown class="text-red-500" size=128 />
                </div>

                <div class="flex flex-col items-center mt-6 space-y-10">
                    <div>
                        <h1 class="text-3xl text-fuente text-center font-bold">{&translations["error_screen_heading"]}</h1>
                        <p class="text-3xl text-fuente text-center font-bold">{&translations["error_screen_text"]}</p>
                    </div>

                    <h3 class="text-fuente font-bold text-2xl">{format!("#{}", &order.order_id()[..8])}</h3>

                    <div class="flex flex-col items-center justify-center">
                        <p class="font-light text-center text-fuente">{&translations["error_screen_detail"]}</p>
                        <button class="bg-fuente-buttons text-fuente-forms py-4 px-7 rounded-full mt-5 font-bold">
                            {&translations["error_screen_detail_message"]}
                        </button>
                    </div>
                </div>
            </main>
        </>
    }
}
