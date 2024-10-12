use yew::prelude::*;

use crate::models::products::{ProductItem, ProductMenu};

#[derive(Properties, Clone, PartialEq)]
pub struct ProductCardProps {
    pub product: ProductItem,
}
#[function_component(ProductCard)]
pub fn product_card(props: &ProductCardProps) -> Html {
    let ProductCardProps { product } = props;
    html! {
        <div class="p-4 shadow-xl rounded-xl w-fit h-fit">
            <div class="w-fit flex flex-row gap-4">
               <div class="w-12 h-12 min-w-12 min-h-12 bg-neutral-300 rounded-full"></div>
               <div class="flex flex-col">
                   <h3 class="text-lg font-bold">{format!("{} - ${}", product.name(), product.price())}</h3>
                   <p class="text-neutral-400">{product.description()}</p>
               </div>
            </div>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct ProductMenuProps {
    pub menu: ProductMenu,
}
#[function_component(ProductMenuCard)]
pub fn product_menu_details(props: &ProductMenuProps) -> Html {
    let ProductMenuProps { menu } = props;
    let menu = menu.categories();
    html! {
        {menu.iter().map(|category| {
           html! {
               <div class="flex flex-col gap-2">
                   <h3 class="text-lg font-bold">{category.name().clone()}</h3>
                   <div class="flex flex-col gap-4">
                    {category.products().iter().map(|product| {
                        html! {
                            <ProductCard product={product.clone()} />
                        }
                    }).collect::<Html>()}
                    </div>
               </div>
           }
        }).collect::<Html>()}
    }
}
