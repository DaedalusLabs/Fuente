use std::rc::Rc;

use fuente::models::{
    ConsumerAddress, ConsumerProfile, OrderRequest, ProductItem, ProductOrder,
    NOSTR_KIND_SERVER_REQUEST, TEST_PUB_KEY,
};
use nostr_minions::key_manager::UserIdentity;
use nostro2::notes::NostrNote;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cart {
    order_id: Option<String>,
    cart_items: ProductOrder,
    current_business: Option<String>,
}

impl Cart {
    pub fn last_sent_order(&self) -> Option<String> {
        self.order_id.clone()
    }
    pub fn can_add_from_business(&self, business_id: &str) -> bool {
        match &self.current_business {
            None => true,
            Some(current_id) => current_id == business_id,
        }
    }
    pub fn cart(&self) -> Vec<ProductItem> {
        self.cart_items.products()
    }
    pub async fn sign_request(
        &self,
        keys: &UserIdentity,
        commerce: String,
        profile: ConsumerProfile,
        address: ConsumerAddress,
    ) -> (String, NostrNote) {
        let new_request = OrderRequest::new(commerce, profile, address, self.cart_items.clone());
        let note = new_request.sign_request(keys).await;
        let content = note.to_string();
        let giftwrap = NostrNote {
            pubkey: keys.get_pubkey().await.unwrap(),
            kind: NOSTR_KIND_SERVER_REQUEST,
            content,
            ..Default::default()
        };
        let giftwrap = keys.sign_nip44(giftwrap, TEST_PUB_KEY.to_string()).await.unwrap();
        (note.id.unwrap(), giftwrap)
    }
    pub fn business_id(&self) -> Option<String> {
        self.current_business.clone()
    }
}

pub enum CartAction {
    SentOrder(String),
    AddProduct(ProductItem, String),
    AddOne(ProductItem),
    RemoveProduct(ProductItem),
    ClearProduct(ProductItem),
    ClearCart,
}

impl Reducible for Cart {
    type Action = CartAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CartAction::AddProduct(product, business_id) => {
                // Update AddProduct to include business_id
                let mut cart_items = self.cart_items.clone();

                // If cart is empty, set the business
                let current_business = match &self.current_business {
                    None => Some(business_id),
                    Some(id) => Some(id.clone()),
                };

                cart_items.add(product);
                Rc::new(Cart {
                    cart_items,
                    current_business,
                    order_id: self.order_id.clone(),
                })
            }
            CartAction::RemoveProduct(product) => {
                let mut cart_items = self.cart_items.clone();
                cart_items.remove_one(product.id());

                // If cart becomes empty, clear business id
                let current_business = if cart_items.is_empty() {
                    None
                } else {
                    self.current_business.clone()
                };

                Rc::new(Cart {
                    cart_items,
                    current_business,
                    order_id: self.order_id.clone(),
                })
            }
            CartAction::AddOne(product) => {
                let mut cart_items = self.cart_items.clone();
                cart_items.add(product);
                Rc::new(Cart {
                    cart_items,
                    current_business: self.current_business.clone(),
                    order_id: self.order_id.clone(),
                })
            }
            CartAction::ClearProduct(product) => {
                let mut cart_items = self.cart_items.clone();
                cart_items.remove_all(product.id());
                Rc::new(Cart {
                    cart_items,
                    current_business: self.current_business.clone(),
                    order_id: self.order_id.clone(),
                })
            }
            CartAction::ClearCart => Rc::new(Cart {
                cart_items: ProductOrder::new(vec![]),
                current_business: None,
                order_id: self.order_id.clone(),
            }),
            CartAction::SentOrder(order_id) => Rc::new(Cart {
                order_id: Some(order_id),
                cart_items: self.cart_items.clone(),
                current_business: self.current_business.clone(),
            }),
        }
    }
}

pub type CartStore = UseReducerHandle<Cart>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct CartChildren {
    pub children: Children,
}

#[function_component(CartProvider)]
pub fn key_handler(props: &CartChildren) -> Html {
    let ctx = use_reducer(|| Cart {
        cart_items: ProductOrder::new(vec![]),
        current_business: None,
        order_id: None,
    });

    html! {
        <ContextProvider<CartStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<CartStore>>
    }
}
