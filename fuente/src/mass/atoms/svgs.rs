use yew::{html, Html};

#[function_component(ThreeBlockSpinner)]
pub fn three_block_shuffle(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <style>
                {r#".spinner_9y7u{animation:spinner_fUkk 2.4s linear infinite;animation-delay:-2.4s}.spinner_DF2s{animation-delay:-1.6s}.spinner_q27e{animation-delay:-.8s}@keyframes spinner_fUkk{8.33%{x:13px;y:1px}25%{x:13px;y:1px}33.3%{x:13px;y:13px}50%{x:13px;y:13px}58.33%{x:1px;y:13px}75%{x:1px;y:13px}83.33%{x:1px;y:1px}}"#}
            </style>
            <rect class="spinner_9y7u" x="1" y="1" rx="1" width="10" height="10"/>
            <rect class="spinner_9y7u spinner_DF2s" x="1" y="1" rx="1" width="10" height="10"/>
            <rect class="spinner_9y7u spinner_q27e" x="1" y="1" rx="1" width="10" height="10"/>
        </svg>
    }
}

use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct IconProps {
    pub class: &'static str,
}

#[function_component(SpinnerIcon)]
pub fn spinner_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg {class} viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <style>
                {".spinner_z9k8{transform-origin:center;animation:spinner_StKS .75s infinite linear}@keyframes spinner_StKS{100%{transform:rotate(360deg)}}"}
            </style>
            <path d="M12,1A11,11,0,1,0,23,12,11,11,0,0,0,12,1Zm0,19a8,8,0,1,1,8-8A8,8,0,0,1,12,20Z" opacity=".25"/>
            <path d="M12,4a8,8,0,0,1,7.89,6.7A1.53,1.53,0,0,0,21.38,12h0a1.5,1.5,0,0,0,1.48-1.75,11,11,0,0,0-21.72,0A1.5,1.5,0,0,0,2.62,12h0a1.53,1.53,0,0,0,1.49-1.3A8,8,0,0,1,12,4Z" class="spinner_z9k8"/>
        </svg>
    }
}

#[function_component(BitcoinIcon)]
pub fn store_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" />
            <path d="M9 8h4.09c1.055 0 1.91 .895 1.91 2s-.855 2 -1.91 2c1.055 0 1.91 .895 1.91 2s-.855 2 -1.91 2h-4.09" />
            <path d="M10 12h4" /><path d="M10 7v10v-9" /><path d="M13 7v1" /><path d="M13 16v1" />
        </svg>
    }
}
