use crate::contexts::{
    cart::{CartAction, CartStore},
    commerce_data::CommerceDataStore,
    consumer_data::ConsumerDataStore,
};

use super::PageHeader;
use fuente::{
    browser_api::HtmlForm,
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    mass::{atoms::layouts::CardComponent, molecules::products::ProductCard},
    models::products::{ProductItem, ProductOrder},
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct CommercePageProps {
    pub commerce_id: String,
}

#[function_component(CommercePage)]
pub fn history_page(props: &CommercePageProps) -> Html {
    let CommercePageProps { commerce_id } = props;
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Consumer context not found");
    let sent_order_request = use_state(|| None::<String>);
    let menu = commerce_ctx
        .products_lists()
        .iter()
        .find(|p| p.id() == *commerce_id)
        .cloned();
    let add_cart = cart_ctx.clone();
    let sender = relay_ctx.send_note.clone();

    let id = commerce_id.clone();
    let profile = user_ctx.get_profile();
    let address = user_ctx.get_default_address();
    let sent_handle = sent_order_request.clone();
    let send_order_request = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        let keys = key_ctx.get_nostr_key();
        let note = cart_ctx.sign_request(
            &keys.unwrap(),
            id.clone(),
            profile.clone().unwrap(),
            address.clone().unwrap(),
        );
        sent_handle.set(Some(note.get_id().to_string()));
        sender.emit(note);
    });

    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Could not capture form");
        let product_str = form.input_value("product").expect("Could not get product");
        let product: ProductItem = product_str.try_into().expect("Could not parse product");
        add_cart.dispatch(CartAction::AddProduct(product));
    });
    if menu.is_none() {
        return html! {
            <div class="h-full w-full flex flex-col">
                <PageHeader title={"Commerce".to_string()} />
                <div class="flex flex-1 flex-col gap-4">
                    <span class="text-lg font-bold">{"No products found"}</span>
                    <span class="text-neutral-400">{"Add some products to your commerce"}</span>
                </div>
            </div>
        };
    }
    let menu = menu.unwrap().menu().categories();
    html! {
        <div class="h-full w-full flex flex-col">
            <PageHeader title={"Commerce".to_string()} />
            <div class="flex flex-1 flex-col gap-4">
                <button
                    class="bg-fuente-light text-white p-2 rounded-md font-mplus m-4"
                    onclick={send_order_request}>
                    {"Send Order Request"}
                </button>
                <CartDetails />
                {menu.iter().map(|category| {
                   html! {
                       <div class="flex flex-col gap-2 px-4">
                           <h3 class="text-lg font-bold">{category.name().clone()}</h3>
                           <div class="flex flex-col gap-4">
                            {category.products().iter().map(|product| {
                                let onsubmit = onsubmit.clone();
                                let product_str = product.to_string();
                                let name = "product";
                                html! {
                                    <form {onsubmit} >
                                        <input class="hidden" {name} value={product_str} />
                                        <button
                                            type="submit">
                                            <ProductCard product={product.clone()} />
                                        </button>
                                    </form>
                                }
                            }).collect::<Html>()}
                            </div>
                       </div>
                   }
                }).collect::<Html>()}
            </div>
        </div>
    }
}

#[function_component(CartDetails)]
pub fn cart_details() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("No cart context found");
    let cart = ProductOrder::new(cart_ctx.cart());
    let counted = cart.counted_products();
    if cart.is_empty() {
        return html! {
            <div class="flex flex-col gap-4 p-4">
                <span class="text-lg font-bold">{"Cart is empty"}</span>
                <span class="text-neutral-400">{"Add some products to your cart"}</span>
            </div>
        };
    }
    html! {
        <div class="p-4" >
            <CardComponent>
              <table class="w-full mb-4">
                <thead>
                  <tr class="border-b border-dashed border-gray-300">
                    <th class="text-left py-2">{"Item"}</th>
                    <th class="text-right py-2">{"Qty"}</th>
                    <th class="text-right py-2">{"Price"}</th>
                  </tr>
                </thead>
                <tbody>
                    {counted.iter().map(|(product, count)| {
                        let subtotal = product.price().parse::<f64>().unwrap() * *count as f64;
                        html! {
                            <tr key={product.id()} class="border-b border-dotted border-gray-200">
                                <td class="py-2">{product.name()}</td>
                                <td class="text-right py-2">{count}</td>
                                <td class="text-right py-2">{format!("{:.2}",subtotal)}</td>
                            </tr>
                        }
                    }).collect::<Html>()}
                </tbody>
              </table>
              <div class="flex justify-between font-bold border-t-2 border-black pt-2">
                <span>{"Total"}</span>
                <span>{format!("{:.2}", cart.total())}</span>
              </div>
            </CardComponent>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct CartItemProp {
    pub product: ProductItem,
    pub count: u32,
}

#[function_component(CartItem)]
pub fn cart_item(props: &CartItemProp) -> Html {
    let CartItemProp { product, count } = props;
    let price = product.price().parse::<f64>().unwrap();
    let total = price * *count as f64;
    html! {
        <CardComponent>
            <div class="flex flex-row gap-4 relative p-4">
                <div class="w-16 h-16 bg-neutral-200 rounded-2xl"></div>
                <div class="flex flex-col">
                    <span class="font-bold text-lg mb-1">{product.name()}</span>
                    <span class="text-neutral-400">{product.price()}</span>
                </div>
                <div class="flex flex-col gap-4">
                    <span class="text-neutral-400">{props.count}</span>
                    <span class="text-neutral-400">{total}</span>
                </div>
            </div>
        </CardComponent>
    }
}
