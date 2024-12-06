use yew::{html, Html};

pub fn notes_svg() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-white h-8 w-8" viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M5 3m0 2a2 2 0 0 1 2 -2h10a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2z" />
            <path d="M9 7l6 0" />
            <path d="M9 11l6 0" />
            <path d="M9 15l4 0" />
        </svg>
    }
}
pub fn cart_svg() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-white h-8 w-8" viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M6 19m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" />
            <path d="M17 19m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" />
            <path d="M17 17h-11v-14h-2" />
            <path d="M6 5l14 1l-1 7h-13" />
        </svg>
    }
}
#[function_component(GearsIcon)]
pub fn gears_svg(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                <path d="M4 10a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" />
                <path d="M6 4v4" />
                <path d="M6 12v8" />
                <path d="M10 16a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" />
                <path d="M12 4v10" />
                <path d="M12 18v2" />
                <path d="M16 7a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" />
                <path d="M18 4v1" />
                <path d="M18 9v11" />
        </svg>
    }
}
#[function_component(LookupIcon)]
pub fn lookup_svg(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <circle cx="10" cy="10" r="7" />
            <line x1="21" y1="21" x2="15" y2="15" />
        </svg>
    }
}

pub fn map_svg() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-white h-8 w-8" viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M12 18.5l-3 -1.5l-6 3v-13l6 -3l6 3l6 -3v7" />
            <path d="M9 4v13" />
            <path d="M15 7v5" />
            <path d="M21.121 20.121a3 3 0 1 0 -4.242 0c.418 .419 1.125 1.045 2.121 1.879c1.051 -.89 1.759 -1.516 2.121 -1.879z" />
            <path d="M19 18v.01" />
        </svg>
    }
}

#[function_component(HomeIcon)]
pub fn home_svg(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M5 12l-2 0l9 -9l9 9l-2 0" />
            <path d="M5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-7" />
            <path d="M9 21v-6a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v6" />
        </svg>
    }
}
pub fn mail_svg() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-white h-8 w-8" viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <rect x="3" y="5" width="18" height="14" rx="2" />
            <polyline points="3 7 12 13 21 7" />
        </svg>
    }
}

pub fn plus_svg() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-green h-8 w-8" viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                <path d="M12 5l0 14" />
                <path d="M5 12l14 0" />
        </svg>
    }
}

pub fn circle_minus_svg() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-red h-8 w-8" viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                <path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" />
                <path d="M9 12l6 0" />
            </svg>
    }
}
use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct IconProps {
    pub class: &'static str,
}

#[function_component(WalletIcon)]
pub fn wallet_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M17 8v-3a1 1 0 0 0 -1 -1h-10a2 2 0 0 0 0 4h12a1 1 0 0 1 1 1v3m0 4v3a1 1 0 0 1 -1 1h-12a2 2 0 0 1 -2 -2v-12" />
            <path d="M20 12v4h-4a2 2 0 0 1 0 -4h4" />
        </svg>
    }
}

#[function_component(MailIcon)]
pub fn mail_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M15 19h-10a2 2 0 0 1 -2 -2v-10a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v4.5" />
            <path d="M19 22v.01" />
            <path d="M19 19a2.003 2.003 0 0 0 .914 -3.782a1.98 1.98 0 0 0 -2.414 .483" />
            <path d="M3 7l9 6l9 -6" />
        </svg>
    }
}

#[function_component(ProfileIcon)]
pub fn profile_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M20 6v12a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2h10a2 2 0 0 1 2 2z" />
            <path d="M10 16h6" />
            <path d="M13 11m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" />
            <path d="M4 8h3" />
            <path d="M4 12h3" />
            <path d="M4 16h3" />
        </svg>
    }
}

#[function_component(CalendarIcon)]
pub fn calendar_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M4 7a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2v-12z" />
            <path d="M16 3v4" />
            <path d="M8 3v4" />
            <path d="M4 11h16" />
            <path d="M7 14h.013" />
            <path d="M10.01 14h.005" />
            <path d="M13.01 14h.005" />
            <path d="M16.015 14h.005" />
            <path d="M13.015 17h.005" />
            <path d="M7.01 17h.005" />
            <path d="M10.01 17h.005" />
        </svg>
    }
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

#[function_component(KeyIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M14 10m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" />
            <path d="M21 12a9 9 0 1 1 -18 0a9 9 0 0 1 18 0z" />
            <path d="M12.5 11.5l-4 4l1.5 1.5" />
            <path d="M12 15l-1.5 -1.5" />
        </svg>
    }
}
#[function_component(ClockIcon)]
pub fn clock_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" />
            <path d="M12 7v5l3 3" />
        </svg>
    }
}
#[function_component(PhoneIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.427 14.768 17.2 13.542a1.733 1.733 0 0 0-2.45 0l-.613.613a1.732 1.732 0 0 1-2.45 0l-1.838-1.84a1.735 1.735 0 0 1 0-2.452l.612-.613a1.735 1.735 0 0 0 0-2.452L9.237 5.572a1.6 1.6 0 0 0-2.45 0c-3.223 3.2-1.702 6.896 1.519 10.117 3.22 3.221 6.914 4.745 10.12 1.535a1.601 1.601 0 0 0 0-2.456Z"/>
        </svg>
    }
}
#[function_component(CheckIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
        </svg>
    }
}
#[function_component(CancelIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m15 9-6 6m0-6 6 6m6-3a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"/>
        </svg>
    }
}
#[function_component(SquareCheckIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M9 12l2 2l4 -4" />
            <path d="M12 3c7.2 0 9 1.8 9 9s-1.8 9 -9 9s-9 -1.8 -9 -9s1.8 -9 9 -9z" />
        </svg>
    }
}
#[function_component(CalendarTimeIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M11.795 21h-6.795a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v4" />
            <path d="M18 18m-4 0a4 4 0 1 0 8 0a4 4 0 1 0 -8 0" />
            <path d="M15 3v4" />
            <path d="M7 3v4" />
            <path d="M3 11h16" />
            <path d="M18 16.496v1.504l1 1" />
        </svg>
    }
}
#[function_component(UserBadgeIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M5 3m0 3a3 3 0 0 1 3 -3h8a3 3 0 0 1 3 3v12a3 3 0 0 1 -3 3h-8a3 3 0 0 1 -3 -3z" />
            <path d="M12 13m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" />
            <path d="M10 6h4" />
            <path d="M9 18h6" />
        </svg>
    }
}
#[function_component(ConfirmationCheckIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24" stroke-width="1.5" stroke="#currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M12.01 2.011a3.2 3.2 0 0 1 2.113 .797l.154 .145l.698 .698a1.2 1.2 0 0 0 .71 .341l.135 .008h1a3.2 3.2 0 0 1 3.195 3.018l.005 .182v1c0 .27 .092 .533 .258 .743l.09 .1l.697 .698a3.2 3.2 0 0 1 .147 4.382l-.145 .154l-.698 .698a1.2 1.2 0 0 0 -.341 .71l-.008 .135v1a3.2 3.2 0 0 1 -3.018 3.195l-.182 .005h-1a1.2 1.2 0 0 0 -.743 .258l-.1 .09l-.698 .697a3.2 3.2 0 0 1 -4.382 .147l-.154 -.145l-.698 -.698a1.2 1.2 0 0 0 -.71 -.341l-.135 -.008h-1a3.2 3.2 0 0 1 -3.195 -3.018l-.005 -.182v-1a1.2 1.2 0 0 0 -.258 -.743l-.09 -.1l-.697 -.698a3.2 3.2 0 0 1 -.147 -4.382l.145 -.154l.698 -.698a1.2 1.2 0 0 0 .341 -.71l.008 -.135v-1l.005 -.182a3.2 3.2 0 0 1 3.013 -3.013l.182 -.005h1a1.2 1.2 0 0 0 .743 -.258l.1 -.09l.698 -.697a3.2 3.2 0 0 1 2.269 -.944zm3.697 7.282a1 1 0 0 0 -1.414 0l-3.293 3.292l-1.293 -1.292l-.094 -.083a1 1 0 0 0 -1.32 1.497l2 2l.094 .083a1 1 0 0 0 1.32 -.083l4 -4l.083 -.094a1 1 0 0 0 -.083 -1.32z" stroke-width="0" fill="currentColor" />
        </svg>
    }
}

#[function_component(CopyIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M9 5h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2h-2" />
            <path d="M9 3m0 2a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v0a2 2 0 0 1 -2 2h-2a2 2 0 0 1 -2 -2z" />
            <path d="M9 14l2 2l4 -4" />
        </svg>
    }
}
#[function_component(BackArrowIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M5 12l14 0" />
            <path d="M5 12l4 4" />
            <path d="M5 12l4 -4" />
        </svg>
    }
}
#[function_component(ArrowBadgeDownIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M17 13v-6l-5 4l-5 -4v6l5 4z" />
        </svg>
    }
}
#[function_component(MenuBarsIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M4 6l16 0" />
            <path d="M4 12l16 0" />
            <path d="M4 18l16 0" />
        </svg>
    }
}
#[function_component(ShoppingCartIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M6 19m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" />
            <path d="M17 19m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" />
            <path d="M17 17h-11v-14h-2" />
            <path d="M6 5l14 1l-1 7h-13" />
        </svg>
    }
}
#[function_component(HistoryIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M12 8l0 4l2 2" />
            <path d="M3.05 11a9 9 0 1 1 .5 4m-.5 5v-5h5" />
        </svg>
    }
}
#[function_component(HeartIcon)]
pub fn key_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M19.5 12.572l-7.5 7.428l-7.5 -7.428a5 5 0 1 1 7.5 -6.566a5 5 0 1 1 7.5 6.572" />
        </svg>
    }
}
#[function_component(ChevronRightIcon)]
pub fn chevron_right_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M9 6l6 6l-6 6" />
        </svg>
    }
}
#[function_component(CategoriesIcon)]
pub fn categories_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M14 4h6v6h-6z" />
            <path d="M4 14h6v6h-6z" />
            <path d="M17 17m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
            <path d="M7 7m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
        </svg>
    }
}
#[function_component(MotoIcon)]
pub fn moto_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M5 16m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
            <path d="M19 16m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
            <path d="M7.5 14h5l4 -4h-10.5m1.5 4l4 -4" />
            <path d="M13 6h2l1.5 3l2 4" />
        </svg>
    }
}
#[function_component(StoreIcon)]
pub fn store_icon(props: &IconProps) -> Html {
    let class = props.class;
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} viewBox="0 0 24 24"
            stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M3 21l18 0" />
            <path d="M3 7v1a3 3 0 0 0 6 0v-1m0 1a3 3 0 0 0 6 0v-1m0 1a3 3 0 0 0 6 0v-1h-18l2 -4h14l2 4" />
            <path d="M5 21l0 -10.15" />
            <path d="M19 21l0 -10.15" />
            <path d="M9 21v-4a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v4" />
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
