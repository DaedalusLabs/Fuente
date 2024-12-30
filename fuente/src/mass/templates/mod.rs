use lucide_yew::SquarePen;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SettingsSideBarBrops {
    pub options: Vec<(String, Callback<MouseEvent>, bool)>,
}

#[function_component(SettingsSideBar)]
pub fn settings_sidebar(props: &SettingsSideBarBrops) -> Html {
    let SettingsSideBarBrops { options } = props;
    let selected_class = classes!("bg-fuente", "text-white");
    let unselected_class = classes!("bg-gray-100", "text-gray-500");
    html! {
        <aside class="flex flex-col gap-3">
            {for options.iter().map(|(name, onclick, selected)| {
                html! {
                    <button
                        type="button"
                        class={classes!(
                            "py-4",
                            "px-8",
                            "rounded-2xl",
                            "font-bold",
                            "text-lg",
                            if *selected { selected_class.clone() } else { unselected_class.clone() }
                        )}
                        {onclick}>
                        {name.as_str()}
                    </button>
                }
            })}
        </aside>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsContentProps {
    pub children: Children,
    pub edit_button: Option<Html>,
}

#[function_component(SettingsContent)]
pub fn settings_content(props: &SettingsContentProps) -> Html {
    let SettingsContentProps {
        children,
        edit_button,
    } = props;
    html! {
        <div class="border border-fuente rounded-xl flex items-start justify-between flex-1 gap-5" style="padding: 40px 50px 40px 80px;">
            <div class="w-full">
                {children}
            </div>
            {edit_button.clone().unwrap_or_default()}

        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsButtonContentProps {
    pub onclick: Callback<MouseEvent>,
}

#[function_component(SettingsContentButton)]
pub fn settings_content_button(props: &SettingsButtonContentProps) -> Html {
    let SettingsButtonContentProps { onclick } = props;
    html! {
        <button type="button" class="flex gap-4 tracking-wide" {onclick}>
            <span class="text-fuente font-bold text-xl">{"Edit"}</span>
            <SquarePen class="w-6 h-6 text-fuente" />
        </button>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct SettingsOptionsButtonsProps {
    pub options: Vec<Html>,
}

#[function_component(SettingsOptionsButtons)]
pub fn settings_options_buttons(props: &SettingsOptionsButtonsProps) -> Html {
    let SettingsOptionsButtonsProps { options } = props;
    html! {
        <div class="flex items-center gap-4">
            {for options.iter().map(|name| {
                html! {
                    {name.clone()}
                }
            })}
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct SettingsPageTemplateProps {
    pub children: Children,
    pub options: Vec<Html>,
    pub heading: String,
    pub content_button: Option<Html>,
    pub sidebar_options: Vec<(String, Callback<MouseEvent>, bool)>,
}

#[function_component(SettingsPageTemplate)]
pub fn settings_template(props: &SettingsPageTemplateProps) -> Html {
    let SettingsPageTemplateProps {
        children,
        options,
        heading,
        sidebar_options,
        content_button,
    } = props.clone();
    html! {
        <>
    <div class="container mx-auto py-10 flex justify-between">
        <h1 class="text-fuente text-6xl font-bold tracking-tighter">{&heading}</h1>
        <SettingsOptionsButtons {options} />
    </div>

    <main class="container mx-auto">
        <div class="flex gap-10">
            <SettingsSideBar options={sidebar_options} />
            <SettingsContent edit_button={content_button} >
                <>
                    {children}
                </>
            </SettingsContent>
        </div>
    </main>
    </>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct LoginPageProps {
    pub children: Children,
    pub heading: String,
    pub sub_heading: String,
    pub title: String,
}

#[function_component(LoginPageTemplate)]
pub fn login_template(props: &LoginPageProps) -> Html {
    let LoginPageProps {
        heading,
        sub_heading,
        title,
        children,
    } = props;
    html! {
    <main class="flex min-h-screen">
        <div class="bg-white bg-logo bg-no-repeat bg-[length:200px_75px] mt-10 ml-16 flex justify-end items-center flex-1">
            <h2 class="text-fuente text-[135px] lg:text-[175px] font-bold -rotate-90 -mr-24 tracking-tighter lg:tracking-[-1rem]">
                {&heading}
            </h2>
        </div>

        <div class="bg-fuente flex items-center flex-auto">
            <h2 class="text-white text-[135px] lg:text-[175px] font-bold -rotate-90 -ml-[5.4rem] lg:-ml-[7.9rem] tracking-[-1rem] -mb-16">
                {&sub_heading}
            </h2>

            <div class="w-full lg:max-w-[360px] mx-auto xl:ml-44 lg:mb-48">
                <p class="text-6xl text-white font-bold -mb-2 relative z-10 text-right tracking-tighter mr-4">
                    {&title}
                </p>
                {children}
            </div>
        </div>
    </main>
    }
}
#[function_component(FuenteBitcoinBanner)]
pub fn bitcoin_banner() -> Html {
    html! {
    <div class="container mx-auto grid gap-5 lg:gap-0 grid-cols-[3fr_1fr] lg:place-items-center">
        <div class="bg-orange-400 w-full mt-10 rounded-2xl h-fit lg:max-h-52">
            <div class="flex items-center">
                <svg viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg" class="w-80 -rotate-12 -mt-5 lg:-mt-14">
                    <path fill="none" d="M0 0h256v256H0z"></path><path d="M184 184H69.8L41.9 30.6a8 8 0 0 0-7.8-6.6H16" fill="none" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></path><circle cx="80" cy="204" fill="none" r="20" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></circle><circle cx="184" cy="204" fill="none" r="20" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></circle><path d="M62.5 144h125.6a15.9 15.9 0 0 0 15.7-13.1L216 64H48" fill="none" stroke="#fcfcfc" stroke-linecap="round" stroke-linejoin="round" stroke-width="16" class="stroke-000000"></path>
                </svg>
                <h2 class="text-3xl lg:text-5xl tracking-tighter text-white font-semibold max-w-[500px] lg:-mt-14 mx-auto">{"Shop now, track your package and pay with"}</h2>
            </div>
        </div>

        <img src="templates/img/bitcoin.png" alt="Bitcoin Logo" class="hidden lg:flex" />
        <svg height="512px" id="svg2" preserveAspectRatio="xMidYMid" version="1.1" viewBox="0 0 1 1" width="512px" xmlns="http://www.w3.org/2000/svg"   class="lg:hidden w-1/2 h-full">
            <defs id="defs4"><filter color-interpolation-filters="sRGB" id="_drop-shadow"><feGaussianBlur id="feGaussianBlur7" in="SourceAlpha" result="blur-out" stdDeviation="1"/><feBlend id="feBlend9" in="SourceGraphic" in2="blur-out" mode="normal"/></filter><linearGradient id="coin-gradient" x1="0%" x2="0%" y1="0%" y2="100%"><stop id="stop12" offset="0%" style="stop-color:#f9aa4b"/><stop id="stop14" offset="100%" style="stop-color:#f7931a"/></linearGradient></defs><g id="g16" transform="scale(0.015625)"><path d="m 63.0359,39.741 c -4.274,17.143 -21.637,27.576 -38.782,23.301 -17.138,-4.274 -27.571,-21.638 -23.295,-38.78 4.272,-17.145 21.635,-27.579 38.775,-23.305 17.144,4.274 27.576,21.64 23.302,38.784 z" id="coin" style="fill:url(#coin-gradient)"/><path d="m 46.1009,27.441 c 0.637,-4.258 -2.605,-6.547 -7.038,-8.074 l 1.438,-5.768 -3.511,-0.875 -1.4,5.616 c -0.923,-0.23 -1.871,-0.447 -2.813,-0.662 l 1.41,-5.653 -3.509,-0.875 -1.439,5.766 c -0.764,-0.174 -1.514,-0.346 -2.242,-0.527 l 0.004,-0.018 -4.842,-1.209 -0.934,3.75 c 0,0 2.605,0.597 2.55,0.634 1.422,0.355 1.679,1.296 1.636,2.042 l -1.638,6.571 c 0.098,0.025 0.225,0.061 0.365,0.117 -0.117,-0.029 -0.242,-0.061 -0.371,-0.092 l -2.296,9.205 c -0.174,0.432 -0.615,1.08 -1.609,0.834 0.035,0.051 -2.552,-0.637 -2.552,-0.637 l -1.743,4.019 4.569,1.139 c 0.85,0.213 1.683,0.436 2.503,0.646 l -1.453,5.834 3.507,0.875 1.439,-5.772 c 0.958,0.26 1.888,0.5 2.798,0.726 l -1.434,5.745 3.511,0.875 1.453,-5.823 c 5.987,1.133 10.489,0.676 12.384,-4.739 1.527,-4.36 -0.076,-6.875 -3.226,-8.515 2.294,-0.529 4.022,-2.038 4.483,-5.155 z m -8.022,11.249 c -1.085,4.36 -8.426,2.003 -10.806,1.412 l 1.928,-7.729 c 2.38,0.594 10.012,1.77 8.878,6.317 z m 1.086,-11.312 c -0.99,3.966 -7.1,1.951 -9.082,1.457 l 1.748,-7.01 c 1.982,0.494 8.365,1.416 7.334,5.553 z" id="symbol" style="fill:#ffffff"/></g>
        </svg>
    </div>
    }
}
#[function_component(FuenteHotCategories)]
pub fn categories_banner() -> Html {
    html! {

    <main class="container mx-auto lg:mt-20 flex flex-col lg:grid lg:grid-cols-[1fr_3fr] lg:gap-5">
        <div class="bg-fuente rounded-2xl p-5 flex flex-col lg:justify-between lg:relative">
            <div class="flex justify-between items-center lg:mb-4">
                <h2 class="text-white text-4xl font-semibold tracking-tighter">{"More Stores"}</h2>
                <svg viewBox="0 0 64 64"  xmlns="http://www.w3.org/2000/svg" class="w-16 h-16">
                    <path d="M4-272.1c-13.2 0-23.9-10.7-23.9-23.9S-9.2-319.9 4-319.9s23.9 10.7 23.9 23.9S17.2-272.1 4-272.1zm0-45.2c-11.7 0-21.3 9.6-21.3 21.3s9.6 21.3 21.3 21.3 21.3-9.6 21.3-21.3-9.6-21.3-21.3-21.3z" transform="translate(28 328)" fill="#ffffff"></path>
                    <path d="m3.5-282.3-1.8-1.9L13.4-296 1.7-307.8l1.8-1.9L17.2-296 3.5-282.3" transform="translate(28 328)" fill="#ffffff"></path>
                    <path d="M15.3-294.6h-24v-2.8h24z" transform="translate(28 328)" fill="#ffffff"></path>
                </svg>
            </div>

            <img src="/templates/img/store.png" alt="Store Image" class="object-contain w-64 mx-auto lg:absolute lg:bottom-0 lg:right-8" />
        </div>
        <div class="overflow-x-auto whitespace-nowrap mt-10 lg:mt-0">
            <div class="flex justify-between items-center">
                <h2 class="text-fuente text-5xl font-bold tracking-tighter">{"Hot Categories!"}</h2>
                <svg viewBox="0 0 64 64"  xmlns="http://www.w3.org/2000/svg" enable-background="new 0 0 64 64" class="w-16 h-16">
                    <path d="M4-272.1c-13.2 0-23.9-10.7-23.9-23.9S-9.2-319.9 4-319.9s23.9 10.7 23.9 23.9S17.2-272.1 4-272.1zm0-45.2c-11.7 0-21.3 9.6-21.3 21.3s9.6 21.3 21.3 21.3 21.3-9.6 21.3-21.3-9.6-21.3-21.3-21.3z" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path><path d="m3.5-282.3-1.8-1.9L13.4-296 1.7-307.8l1.8-1.9L17.2-296 3.5-282.3" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path><path d="M15.3-294.6h-24v-2.8h24z" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path>
                </svg>
            </div>

            <div class=" mt-10">
                <div class="grid grid-flow-col auto-cols-max gap-4">
                    <div class="bg-fuente-light rounded-2xl flex items-center w-80">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{"Books"}</p>
                    </div>
                    <img src="/templates/img/iphone.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/sneaker_1.png" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/yumi.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <div class="bg-fuente-light rounded-2xl w-80 flex items-center">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{"Tech"}</p>
                    </div>
                    <img src="/templates/img/iphone.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/sneaker_1.png" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/yumi.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                </div>

                <div class="grid grid-flow-col auto-cols-max gap-4 mt-5">
                    <img src="/templates/img/ninja.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <div class="bg-fuente-light rounded-2xl flex items-center w-80">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{"Movies"}</p>
                    </div>
                    <img src="/templates/img/candy.jpg" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/whey.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/ninja.png" alt="iPhone Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <div class="bg-fuente-light rounded-2xl w-80 flex items-center">
                        <p class="text-white text-6xl font-bold tracking-tighter text-center flex-1">{"Music"}</p>
                    </div>
                    <img src="/templates/img/candy.jpg" alt="Sneaker Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                    <img src="/templates/img/whey.jpg" alt="Yumi Product" class="max-w-32 bg-gray-100 w-full object-cover flex h-full p-5 rounded-2xl" />
                </div>
            </div>
        </div>
    </main>
    }
}
#[function_component(FuenteSalesPitch)]
pub fn sales_pitch() -> Html {
    html! {
    <div class="flex flex-col justify-center lg:flex-row lg:justify-between items-center lg:relative bg-sky-200 rounded-2xl px-10 py-5 container mx-auto mt-7">
        <div class="flex">
            <img src="templates/img/sneaker_1.png" alt="Sneaker Product" class="lg:absolute object-contain w-32 lg:w-40 mt-10 xl:mt-4 xl:w-52 top-0 left-0 z-10" />
            <img src="templates/img/sneaker_2.png" alt="Sneaker Product" class="lg:absolute object-contain w-32 lg:w-40 mt-10 xl:mt-4 xl:w-52 -top-10 left-28 z-20" />
            <img src="templates/img/sneaker_3.png" alt="Sneaker Product" class="lg:absolute object-contain w-32 lg:w-40 mt-10 xl:mt-4 xl:w-64 lg:top-0 lg:left-56 z-30 hidden lg:flex"/>
        </div>

        <div class="mx-auto lg:mx-0 lg:ml-auto">
            <h2 class="text-5xl text-fuente tracking-tighter font-semibold max-w-[500px] text-center lg:text-left">{"Are you looking for sale your products?"}</h2>
            <div class="flex justify-center lg:justify-start">
                <button class="text-fuente-forms bg-fuente-buttons py-3 px-10 rounded-full font-semibold mt-5">{"Get more info"}</button>
            </div>
        </div>
    </div>
    }
}
