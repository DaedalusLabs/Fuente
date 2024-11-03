mod nostr_relay;
mod relay_pool;
pub use nostr_relay::*;
pub use relay_pool::*;

#[yew::function_component(RelayPoolTest)]
pub fn relay_pool_test() -> yew::Html {
    let relay_ctx = yew::use_context::<relay_pool::NostrProps>().expect("No relay context found");
    let subscription_id = yew::use_state(|| None);
    let latest_note = yew::use_state(|| None);

    let subscriber = relay_ctx.subscribe.clone();
    let id_handle = subscription_id.clone();
    yew::use_effect_with((), move |_| {
        let nostr_sub = nostro2::relays::NostrFilter::default()
            .new_kind(20001)
            .subscribe();
        let kind_one_filter = nostro2::relays::NostrFilter::default()
            .new_kind(1)
            .new_limit(20)
            .subscribe();
        id_handle.set(Some(nostr_sub.id()));
        subscriber.emit(nostr_sub);
        subscriber.emit(kind_one_filter);
        || {}
    });
    let note_handle = latest_note.clone();
    let note_counter = yew::use_state(|| 0);

    let counter_handle = note_counter.clone();
    yew::use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            if note.get_kind() == 20001 {
                note_handle.set(Some(note.clone()));
            }
            if note.get_kind() == 1 {
                counter_handle.set(*counter_handle + 1);
            }
        }
        || {}
    });

    let note_sender = relay_ctx.send_note.clone();
    let send_note_onclick = yew::Callback::from(move |_| {
        let new_keys = nostro2::userkeys::UserKeys::generate();
        let timestamp = nostro2::utils::get_unix_timestamp();
        let new_note = nostro2::notes::Note::new(
            &new_keys.get_public_key(),
            20001,
            &format!("Minion Note at {}", timestamp),
        );
        let signed_note = new_keys.sign_nostr_event(new_note);
        note_sender.emit(signed_note);
    });

    match subscription_id.as_ref() {
        Some(id) => {
            let unsubscriber = relay_ctx.unsubscribe.clone();
            let sub_id = id.clone();
            let unsubscribe_onclick = yew::Callback::from(move |_| {
                unsubscriber.emit(sub_id.clone());
            });
            yew::html! {
                <div class="flex flex-col gap-4">
                    <h2 class="text-xl font-bold">{"Relay Pool Test"}</h2>
                    <div class="flex flex-row gap-2">
                        <button onclick={send_note_onclick}>
                            { "Send Note" }
                        </button>
                        <button onclick={unsubscribe_onclick}>
                            { "Unsubscribe" }
                        </button>
                    </div>
                    <div>
                        <h3>{"Kind 1 Count"}</h3>
                        <p>{*note_counter}</p>
                    </div>
                    {{
                        match latest_note.as_ref() {
                            Some(note) => yew::html! {
                                <div>
                                    <h3>{"My Latest Note"}</h3>
                                    <p>{note.get_content()}</p>
                                </div>
                            },
                            None => yew::html! { <div>{"Send a note!"}</div> },
                        }
                    }}
                </div>
            }
        }
        None => yew::html! { <div>{"Loading Relay Pool..."}</div> },
    }
}
