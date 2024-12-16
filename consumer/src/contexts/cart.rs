use std::rc::Rc;

use fuente::models::{
    ConsumerAddress, ConsumerProfile, OrderRequest, ProductItem, ProductOrder, TEST_PUB_KEY,
};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cart {
    cart_items: ProductOrder,
    current_business: Option<String>,
}

impl Cart {
    pub fn can_add_from_business(&self, business_id: &str) -> bool {
        match &self.current_business {
            None => true,
            Some(current_id) => current_id == business_id
        }
    }
    pub fn cart(&self) -> Vec<ProductItem> {
        self.cart_items.products()
    }
    pub fn sign_request(
        &self,
        keys: &NostrKeypair,
        commerce: String,
        profile: ConsumerProfile,
        address: ConsumerAddress,
    ) -> NostrNote {
        let new_request = OrderRequest::new(commerce, profile, address, self.cart_items.clone());
        new_request
            .giftwrapped_request(keys, TEST_PUB_KEY.to_string())
            .unwrap()
    }
}

pub enum CartAction {
    AddProduct(ProductItem, String),
    RemoveProduct(ProductItem),
    ClearCart,
}

impl Reducible for Cart {
    type Action = CartAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CartAction::AddProduct(product, business_id) => { // Update AddProduct to include business_id
                let mut cart_items = self.cart_items.clone();
                
                // If cart is empty, set the business
                let current_business = match &self.current_business {
                    None => Some(business_id),
                    Some(id) => Some(id.clone())
                };

                cart_items.add(product);
                Rc::new(Cart { cart_items, current_business })
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

                Rc::new(Cart { cart_items, current_business })
            }
            CartAction::ClearCart => Rc::new(Cart {
                cart_items: ProductOrder::new(vec![]),
                current_business: None,
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
    });

    html! {
        <ContextProvider<CartStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<CartStore>>
    }
}
