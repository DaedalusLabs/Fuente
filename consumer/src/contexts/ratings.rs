use fuente::models::{ParticipantRating, TEST_PUB_KEY};
use nostr_minions::relay_pool::NostrProps;
use nostro2::relays::NostrSubscription;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RatingsData {
    has_loaded: bool,
    ratings: Vec<ParticipantRating>,
}

impl RatingsData {
    pub fn is_loaded(&self) -> bool {
        self.has_loaded
    }

    pub fn get_rating(&self, pubkey: &str) -> Option<&ParticipantRating> {
        self.ratings.iter().find(|r| r.pubkey == pubkey)
    }

    pub fn get_business_rating(&self, pubkey: &str) -> Option<ParticipantRating> {
        self.ratings
            .iter()
            .find(|r| {
                r.pubkey == pubkey && r.participant == fuente::models::OrderParticipant::Commerce
            })
            .cloned()
    }
}

pub enum RatingsAction {
    SetLoaded,
    UpdateRating(ParticipantRating),
    LoadRatings(Vec<ParticipantRating>),
}

impl Reducible for RatingsData {
    type Action = RatingsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            RatingsAction::SetLoaded => Rc::new(RatingsData {
                has_loaded: true,
                ratings: self.ratings.clone(),
            }),
            RatingsAction::UpdateRating(rating) => {
                let mut ratings = self.ratings.clone();
                if let Some(idx) = ratings.iter().position(|r| r.pubkey == rating.pubkey) {
                    ratings[idx] = rating;
                } else {
                    ratings.push(rating);
                }
                Rc::new(RatingsData {
                    has_loaded: self.has_loaded,
                    ratings,
                })
            }
            RatingsAction::LoadRatings(ratings) => Rc::new(RatingsData {
                has_loaded: self.has_loaded,
                ratings,
            }),
        }
    }
}

pub type RatingsStore = UseReducerHandle<RatingsData>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct RatingsProviderProps {
    pub children: Children,
}

#[function_component(RatingsProvider)]
pub fn ratings_provider(props: &RatingsProviderProps) -> Html {
    let ctx = use_reducer(|| RatingsData {
        has_loaded: false,
        ratings: vec![],
    });

    html! {
        <ContextProvider<RatingsStore> context={ctx}>
            {props.children.clone()}
            <RatingsSync />
        </ContextProvider<RatingsStore>>
    }
}

#[function_component(RatingsSync)]
fn ratings_sync() -> Html {
    let ctx = use_context::<RatingsStore>().expect("RatingsStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe.clone();
    let id_handle = sub_id.clone();

    use_effect_with((), move |_| {
        let filter: nostro2::relays::SubscribeEvent = NostrSubscription {
            kinds: Some(vec![fuente::models::NOSTR_KIND_PARTICIPANT_RATING]),
            authors: Some(vec![TEST_PUB_KEY.to_string()]), // Add this line
            ..Default::default()
        }
        .into();

        id_handle.set(filter.1.clone());
        subscriber.emit(filter);
        || {}
    });

    let ctx_clone = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            if note.kind == fuente::models::NOSTR_KIND_PARTICIPANT_RATING {
                match ParticipantRating::try_from(note.clone()) {
                    Ok(rating) => {
                        ctx_clone.dispatch(RatingsAction::UpdateRating(rating));
                    }
                    Err(e) => {
                        gloo::console::error!("Failed to parse rating:", e.to_string());
                    }
                }
            }
        }
        || {}
    });

    //new effect after the subscription effect:
    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_ctx.relay_events.clone(), move |events| {
        if let Some(event) = events.last() {
            if let nostro2::relays::RelayEvent::EndOfSubscription((_, id)) = event {
                if id == &(*id_handle) {
                    ctx_clone.dispatch(RatingsAction::SetLoaded);
                }
            }
        }
        || {}
    });

    html! {}
}
