use crate::widgets::toastify::ToastifyOptions;
use std::collections::HashMap;

use async_channel::{unbounded, Sender};
use nostro2::{
    notes::SignedNote,
    relays::{NostrSubscription, RelayEvents},
};

use wasm_bindgen::JsValue;
use yew::platform::spawn_local;
use yew::{prelude::*, props};

use super::nostr_relay::UserRelay;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct RelayContextProps {
    pub children: Children,
    pub relays: Vec<UserRelay>,
}

pub enum RelayAction {
    Event(RelayEvents),
    UniqueNote(SignedNote),
    SendNote(SignedNote),
    Subscribe(NostrSubscription),
    Unsubscribe(String),
    Close,
}

#[derive(Properties, Clone, PartialEq)]
pub struct NostrProps {
    pub relay_events: Vec<RelayEvents>,
    pub unique_notes: Vec<SignedNote>,
    pub send_note: Callback<SignedNote>,
    pub subscribe: Callback<NostrSubscription>,
    pub unsubscribe: Callback<String>,
    pub close: Callback<()>,
}
pub struct RelayProvider {
    relay_events: Vec<RelayEvents>,
    unique_notes: Vec<SignedNote>,
    sender_channel: Sender<SignedNote>,
    filter_channel: Sender<NostrSubscription>,
    unsubscribe_channel: Sender<String>,
    close_channel: Sender<()>,
    send_note_callback: Callback<SignedNote>,
    subscribe_callback: Callback<NostrSubscription>,
    unsubscribe_callback: Callback<String>,
    close_callback: Callback<()>,
    children: Children,
}

impl Component for RelayProvider {
    type Message = RelayAction;
    type Properties = RelayContextProps;

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let props = self.build_props();
        html! {
            <>
                <ContextProvider<NostrProps> context={props}>
                    {self.children.clone()}
                </ContextProvider<NostrProps>>
            </>
        }
    }

    fn create(ctx: &Context<Self>) -> Self {
        let relays = ctx.props().relays.clone();
        let (sender_channel, filter_channel, unsubscribe_channel, close_channel) =
            Self::read_relays(
                ctx.link().callback(RelayAction::Event),
                ctx.link().callback(RelayAction::UniqueNote),
                relays,
            );
        let send_note_callback = ctx.link().callback(RelayAction::SendNote);
        let close_callback = ctx.link().callback(move |_| RelayAction::Close);
        let subscribe_callback = ctx.link().callback(RelayAction::Subscribe);
        let unsubscribe_callback = ctx.link().callback(RelayAction::Unsubscribe);
        let children = ctx.props().children.clone();
        let relay_events = Vec::new();
        let unique_notes = Vec::new();

        Self {
            relay_events,
            unique_notes,
            sender_channel,
            close_channel,
            filter_channel,
            unsubscribe_channel,
            send_note_callback,
            close_callback,
            subscribe_callback,
            unsubscribe_callback,
            children,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RelayAction::SendNote(note) => match self.send_nostr_note(note) {
                Ok(_) => true,
                Err(e) => {
                    gloo::console::error!("Error sending note: {:?}", e);
                    false
                }
            },
            RelayAction::Subscribe(filter) => match self.subscribe(filter) {
                Ok(_) => true,
                Err(e) => {
                    gloo::console::error!("Error subscribing: {:?}", e);
                    false
                }
            },
            RelayAction::Close => {
                match self.close_ws() {
                    Ok(_) => (),
                    Err(e) => gloo::console::error!("Error closing websocket: {:?}", e),
                }
                false
            }
            RelayAction::Event(event) => {
                if let RelayEvents::EVENT(_, ref _note) = event {
                    // Add notification for new event.
                    ToastifyOptions::new_event_received("note").show();
                }
                self.add_event(event);
                true
            }
            RelayAction::UniqueNote(note) => {
                self.add_unique_note(note);
                true
            }
            RelayAction::Unsubscribe(filter) => match self.unsubscribe(filter) {
                Ok(_) => true,
                Err(e) => {
                    gloo::console::error!("Error unsubscribing: {:?}", e);
                    false
                }
            },
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        match self.close_ws() {
            Ok(_) => (),
            Err(e) => gloo::console::error!("Error closing websocket: {:?}", e),
        }
    }
}

impl RelayProvider {
    fn read_relays(
        event_cb: Callback<RelayEvents>,
        note_cb: Callback<SignedNote>,
        relays: Vec<UserRelay>,
    ) -> (
        Sender<SignedNote>,
        Sender<NostrSubscription>,
        Sender<String>,
        Sender<()>,
    ) {
        let (send_note_tx, send_note_rx) = unbounded::<SignedNote>();
        let (filter_tx, filter_rx) = unbounded::<NostrSubscription>();
        let (unsubscribe_tx, unsubscribe_rx) = unbounded::<String>();
        let (close_tx, close_rx) = unbounded::<()>();
        
        spawn_local(async move {
            // Show initial connection attempt
            ToastifyOptions::new_relay_connected("Connecting to relay pool").show();
            
            let relay_pool = match nostro2::pool::RelayPool::new(
                relays.iter().map(|relay| relay.url.clone()).collect(),
            ).await {
                Ok(pool) => {
                    ToastifyOptions::new_relay_connected("Connected to relay pool").show();
                    pool
                },
                Err(e) => {
                    ToastifyOptions::new_relay_error(&format!("Failed to create relay pool: {}", e))
                        .show();
                    return;
                }
            };
    
            let pooled_notes = relay_pool.pooled_notes();
            let relay_events = relay_pool.all_events();
            
            loop {
                tokio::select! {
                    event = relay_events.recv() => {
                        if let Ok(event) = event {
                            event_cb.emit(event);
                        }
                    }
                    note = pooled_notes.recv() => {
                        if let Ok(event) = note {
                            note_cb.emit(event);
                            // Show notification for new note
                            ToastifyOptions::new_event_received("note").show();
                        }
                    }
                    note = send_note_rx.recv() => {
                        if let Ok(note) = note {
                            if let Err(e) = relay_pool.broadcast_note(note).await {
                                ToastifyOptions::new_relay_error(&format!("Error broadcasting note: {}", e))
                                    .show();
                            }
                        }
                    }
                    filter = filter_rx.recv() => {
                        if let Ok(filter) = filter {
                            if let Err(e) = relay_pool.subscribe(filter).await {
                                ToastifyOptions::new_relay_error(&format!("Error subscribing: {}", e))
                                    .show();
                            }
                        }
                    }
                    unsubscribe = unsubscribe_rx.recv() => {
                        if let Ok(filter) = unsubscribe {
                            if let Err(e) = relay_pool.cancel_subscription(filter).await {
                                ToastifyOptions::new_relay_error(&format!("Error unsubscribing: {}", e))
                                    .show();
                            }
                        }
                    }
                    _ = close_rx.recv() => {
                        ToastifyOptions::new_relay_disconnected("Disconnecting from relay pool").show();
                        let _ = relay_pool.close().await;
                        break;
                    }
                }
            }
            relay_pool.close().await.unwrap();
        });
        
        (send_note_tx, filter_tx, unsubscribe_tx, close_tx)
    }

    pub fn build_props(&self) -> NostrProps {
        let _unique_notes = self
            .relay_events
            .iter()
            .filter_map(|event| match event {
                RelayEvents::EVENT(_, note) => Some(note.clone()),
                _ => None,
            })
            .fold(HashMap::new(), |mut acc, note| {
                acc.insert(note.get_id().to_string(), note);
                acc
            });
        props!(NostrProps {
            relay_events: self.relay_events.clone(),
            unique_notes: self.unique_notes.clone(),
            send_note: self.send_note_callback.clone(),
            subscribe: self.subscribe_callback.clone(),
            unsubscribe: self.unsubscribe_callback.clone(),
            close: self.close_callback.clone(),
        })
    }

    fn send_nostr_note(&self, signed_note: SignedNote) -> Result<(), JsValue> {
        let _ = self
            .sender_channel
            .try_send(signed_note)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    fn subscribe(&self, filter: NostrSubscription) -> Result<(), JsValue> {
        let _ = self
            .filter_channel
            .try_send(filter)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    fn unsubscribe(&self, filter: String) -> Result<(), JsValue> {
        let _ = self
            .unsubscribe_channel
            .try_send(filter)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    fn add_event(&mut self, event: RelayEvents) {
        self.relay_events.push(event);
    }
    fn add_unique_note(&mut self, note: SignedNote) {
        self.unique_notes.push(note);
    }

    fn close_ws(&self) -> Result<(), JsValue> {
        let _ = self
            .close_channel
            .try_send(())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
}
