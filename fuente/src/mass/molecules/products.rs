use yew::prelude::*;

use crate::models::{ProductItem, ProductMenu, ProductOrder};

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

#[derive(Properties, Clone, PartialEq)]
pub struct ProductMenuListProps {
    pub order: ProductOrder,
}
#[function_component(OrderRequestDetailsComponent)]
pub fn order_request_details(props: &ProductMenuListProps) -> Html {
    let ProductMenuListProps { order } = props;
    let counted = order.counted_products();
    let total = order.total();
    let products_html = html! {
        {counted.iter().map(|(item, count)| {
            html! {
                    <div class="flex flex-row gap-2">
                        <p>{format!("{} x {}", count, item.name())}</p>
                        <p>{format!("{}", item.price().parse::<u32>().unwrap() * count)}</p>
                    </div>
            }
        }).collect::<Html>()}
    };
    html! {
        <div class="flex flex-col gap-4">
            {products_html}
            <div class="flex flex-row justify-between">
                <p class="text-lg font-bold">{"Total"}</p>
                <p class="text-lg font-bold">{format!("${}", total)}</p>
            </div>
        </div>
    }
}
