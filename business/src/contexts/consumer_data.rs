use fuente::{
    browser_api::IdbStoreManager,
    models::{ConsumerAddress, ConsumerAddressIdb, ConsumerProfile, ConsumerProfileIdb},
};
use std::rc::Rc;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumerData {
    has_loaded: (bool, bool),
    profile: Vec<ConsumerProfileIdb>,
    addresses: Vec<ConsumerAddressIdb>,
}

impl ConsumerData {
    pub fn finished_loading(&self) -> bool {
        self.has_loaded == (true, true)
    }
    pub fn get_profiles(&self) -> Vec<ConsumerProfile> {
        self.profile.clone().iter().map(|p| p.profile()).collect()
    }
    pub fn get_addresses(&self) -> Vec<ConsumerAddress> {
        self.addresses.clone().iter().map(|a| a.address()).collect()
    }
    pub fn get_consumer_entries(&self) -> Vec<ConsumerProfileIdb> {
        self.profile.clone()
    }
    pub fn get_address_entries(&self) -> Vec<ConsumerAddressIdb> {
        self.addresses.clone()
    }
}

pub enum ConsumerDataAction {
    FinishedLoadingDb,
    FinishedLoadingRelays,
    LoadProfile(Vec<ConsumerProfileIdb>),
    LoadAddresses(Vec<ConsumerAddressIdb>),
    SetDefaultAddress(ConsumerAddressIdb),
    NewAddress(ConsumerAddressIdb),
    DeleteAddress(ConsumerAddressIdb),
    NewProfile(ConsumerProfileIdb),
    DeleteProfile(ConsumerProfileIdb),
}

impl Reducible for ConsumerData {
    type Action = ConsumerDataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ConsumerDataAction::LoadProfile(profile) => Rc::new(ConsumerData {
                has_loaded: self.has_loaded,
                profile,
                addresses: self.addresses.clone(),
            }),
            ConsumerDataAction::FinishedLoadingDb => Rc::new(ConsumerData {
                has_loaded: (true, true),
                profile: self.profile.clone(),
                addresses: self.addresses.clone(),
            }),
            ConsumerDataAction::LoadAddresses(addresses) => Rc::new(ConsumerData {
                has_loaded: self.has_loaded,
                profile: self.profile.clone(),
                addresses,
            }),
            ConsumerDataAction::FinishedLoadingRelays => Rc::new(ConsumerData {
                has_loaded: (self.has_loaded.0, true),
                profile: self.profile.clone(),
                addresses: self.addresses.clone(),
            }),
            ConsumerDataAction::NewAddress(address) => {
                let db_entry = address.clone();
                spawn_local(async move {
                    db_entry
                        .save_to_store()
                        .await
                        .expect("Failed to save address");
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: self.profile.clone(),
                    addresses: {
                        let mut addresses = self.addresses.clone();
                        addresses.push(address);
                        addresses
                    },
                })
            }
            ConsumerDataAction::DeleteAddress(address) => {
                let db_entry = address.clone();
                spawn_local(async move {
                    let _ = db_entry.delete_from_store().await;
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: self.profile.clone(),
                    addresses: {
                        let mut addresses = self.addresses.clone();
                        addresses.retain(|a| a.id() != address.id());
                        addresses
                    },
                })
            }
            ConsumerDataAction::SetDefaultAddress(address) => {
                let db_entry = address.clone();
                spawn_local(async move {
                    let _ = db_entry.set_as_default().await;
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: self.profile.clone(),
                    addresses: {
                        let mut addresses = self.addresses.clone();
                        addresses.iter_mut().for_each(|a| {
                            if a.id() == address.id() {
                                a.set_default(true);
                            } else {
                                a.set_default(false);
                            }
                        });
                        addresses
                    },
                })
            }
            ConsumerDataAction::NewProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    db_entry
                        .save_to_store()
                        .await
                        .expect("Failed to save profile");
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: {
                        let mut profiles = self.profile.clone();
                        profiles.push(profile);
                        profiles
                    },
                    addresses: self.addresses.clone(),
                })
            }
            ConsumerDataAction::DeleteProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    let _ = db_entry.delete_from_store().await;
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: {
                        let mut profiles = self.profile.clone();
                        profiles.retain(|p| p.pubkey() != profile.pubkey());
                        profiles
                    },
                    addresses: self.addresses.clone(),
                })
            }
        }
    }
}

pub type ConsumerDataStore = UseReducerHandle<ConsumerData>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct ConsumerDataChildren {
    pub children: Children,
}

#[function_component(ConsumerDataProvider)]
pub fn key_handler(props: &ConsumerDataChildren) -> Html {
    let ctx = use_reducer(|| ConsumerData {
        has_loaded: (false, false),
        addresses: vec![],
        profile: vec![],
    });

    let ctx_clone = ctx.clone();
    use_effect_with((), |_| {
        spawn_local(async move {
            if let Ok(profile) = ConsumerProfileIdb::retrieve_all_from_store().await {
                ctx_clone.dispatch(ConsumerDataAction::LoadProfile(profile));
            }
            if let Ok(addresses) = ConsumerAddressIdb::retrieve_all_from_store().await {
                ctx_clone.dispatch(ConsumerDataAction::LoadAddresses(addresses));
            }
            ctx_clone.dispatch(ConsumerDataAction::FinishedLoadingDb);
        });
        || {}
    });

    html! {
        <ContextProvider<ConsumerDataStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<ConsumerDataStore>>
    }
}
