use business::{
    contexts::{CommerceDataProvider, CommerceDataStore, ConsumerDataProvider, OrderDataProvider},
    pages::NewProfilePage,
    router::CommercePages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore, LanguageConfigsProvider},
    mass::{LoadingScreen, LoginPage, ToastProvider,  PwaInstall},
    models::{init_commerce_db, init_consumer_db},
};
use html::ChildrenProps;
use lucide_yew::Smile;
use nostr_minions::{
    init_nostr_db,
    key_manager::{NostrIdProvider, NostrIdStore},
    relay_pool::{RelayProvider, UserRelay},
};
use yew::{platform::spawn_local, prelude::*};
use yew_router::BrowserRouter;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    use_effect_with((), move |_| {
        init_commerce_db().unwrap();
        init_consumer_db().unwrap();
        init_nostr_db().unwrap();
        spawn_local(async move {
            let sw = nostr_minions::browser_api::AppServiceWorker::new()
                .expect("Error initializing service worker");
            sw.install("serviceWorker.js")
                .await
                .expect("Error installing service worker");
        });
        || {}
    });
    html! {
        <LanguageConfigsProvider>
            <PwaInstall/>
            <BrowserRouter>
                <RelayProviderComponent>
                    <AppContext>
                        <ToastProvider>
                            <LoginCheck>
                                <CommercePages />
                            </LoginCheck>
                        </ToastProvider>
                    </AppContext>
                </RelayProviderComponent>
            </BrowserRouter>
        </LanguageConfigsProvider>
    }
}

#[function_component(RelayProviderComponent)]
fn relay_pool_component(props: &ChildrenProps) -> Html {
    let relays = include_str!("../../relays.txt")
        .trim()
        .lines()
        .map(|url| UserRelay {
            url: url.trim().to_string(),
            read: true,
            write: true,
        })
        .collect::<Vec<UserRelay>>();
    html! {
        <RelayProvider {relays}>
            {props.children.clone()}
        </RelayProvider>
    }
}

#[function_component(AppContext)]
fn app_context(props: &ChildrenProps) -> Html {
    html! {
        <NostrIdProvider>
            <AdminConfigsProvider >
                <CommerceDataProvider>
                    <ConsumerDataProvider >
                        <OrderDataProvider>
                            {props.children.clone()}
                        </OrderDataProvider>
                    </ConsumerDataProvider>
                </CommerceDataProvider>
            </AdminConfigsProvider>
        </NostrIdProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let user_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    let config_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    if !key_ctx.loaded() || !config_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    if key_ctx.get_identity().is_none() {
        return html! { <LoginPage /> };
    }
    if !user_ctx.checked_db() {
        return html! {<LoadingScreen />};
    }
    if user_ctx.profile().is_none() && user_ctx.checked_relay() {
        return html! {
            <NewProfilePage />
        };
    }
    let whitelist = config_ctx.get_commerce_whitelist();
    if !whitelist.contains(&key_ctx.get_pubkey().unwrap()) {
        return html! {
            <main class="grid lg:grid-cols-[1fr_3fr] min-h-screen">
                <div class="bg-white hidden lg:block">
                    <img src="/public/assets/img/logo.jpg" class="block object-contain w-56 mx-auto py-10"/>
                </div>
                <div class="bg-fuente w-full flex justify-center items-center">
                    <div class="lg:bg-fuente-forms px-10 py-20 flex-1 rounded-2xl max-w-4xl">
                        <Smile class="text-white size-52 mx-auto" />
                        <h1 class="text-4xl text-center text-white font-bold my-5">{"The application to register your"} <span class="block">{"business was received successfully!"}</span></h1>
                        <p class="text-white text-center text-xl font-thin">{"Currently you aren't authorized to this page."}</p>
                        <p class="text-white text-center text-xl font-thin">{"Please, be patient someone of your team will contact"} <span class="block">{"you to continue with the process."}</span></p>
                    </div>
                </div>
            </main>
        };
    }
    html! {
        {props.children.clone()}
    }
}
