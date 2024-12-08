use std::rc::Rc;

use fuente::models::{
    ConsumerAddress, ConsumerProfile, OrderRequest, ProductItem, ProductOrder, TEST_PUB_KEY,
};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cart {
    cart_items: ProductOrder,
}

impl Cart {
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
    AddProduct(ProductItem),
    RemoveProduct(ProductItem),
    ClearCart,
}

impl Reducible for Cart {
    type Action = CartAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CartAction::AddProduct(product) => {
                let mut cart_items = self.cart_items.clone();
                cart_items.add(product);
                Rc::new(Cart { cart_items })
            }
            CartAction::RemoveProduct(product) => {
                let mut cart_items = self.cart_items.clone();
                cart_items.remove_one(product.id());
                Rc::new(Cart { cart_items })
            }
            CartAction::ClearCart => Rc::new(Cart {
                cart_items: ProductOrder::new(vec![]),
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
    });

    html! {
        <ContextProvider<CartStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<CartStore>>
    }
}
