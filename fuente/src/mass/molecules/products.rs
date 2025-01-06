use yew::prelude::*;

use crate::models::{ProductItem, ProductMenu, ProductOrder};

#[derive(Properties, Clone, PartialEq)]
pub struct ProductCardProps {
    pub product: ProductItem,
    #[prop_or_default]
    pub on_edit: Option<Callback<MouseEvent>>,  // Changed from Callback to Option<Callback>
    #[prop_or_default]
    pub on_delete: Option<Callback<MouseEvent>>, // Changed from Callback to Option<Callback>
    #[prop_or_default]
    pub show_admin_controls: bool,  // New prop
}
#[function_component(ProductCard)]
pub fn product_card(props: &ProductCardProps) -> Html {
    let ProductCardProps { 
        product, 
        on_edit, 
        on_delete,
        show_admin_controls 
    } = props;
    html! {
        <div class="p-4 shadow-xl rounded-xl w-fit h-fit">
            <div class="w-fit flex flex-row gap-4">
                // Add image 
                <img 
                    src={product.image_url()} 
                    alt={product.name()}
                    class="w-12 h-12 min-w-12 min-h-12 bg-neutral-300 rounded-full object-cover"
                />
                <img 
                    src={product.thumbnail_url()} 
                    alt={product.name()}
                    class="w-12 h-12 min-w-12 min-h-12 bg-neutral-300 rounded-full object-cover"
                />
                <div class="flex flex-col">
                    <h3 class="text-lg font-bold">{format!("{} - SRD {}", product.name(), product.price())}</h3>
                    if let Some(discount) = product.discount() {
                        <p class="text-sm text-green-500">
                            {format!("Discount: SRD {}", discount)}
                        </p>
                    }
                    <p class="text-sm text-gray-500">{format!("SKU: {}", product.sku())}</p>
                    <p class="text-neutral-400">{product.description()}</p>
                    <details class="mt-2">
                        <summary class="text-sm font-semibold">{"Product Details"}</summary>
                        <p class="text-sm text-neutral-600 mt-1">{product.details()}</p>
                    </details>
                    if *show_admin_controls {
                        <div class="flex gap-2 mt-2">
                            if let Some(edit_cb) = on_edit.clone() {
                                <button 
                                    onclick={edit_cb}
                                    class="text-sm text-blue-500 hover:text-blue-700">
                                    {"Edit"}
                                </button>
                            }
                            if let Some(delete_cb) = on_delete.clone() {
                                <button 
                                    onclick={delete_cb}
                                    class="text-sm text-red-500 hover:text-red-700">
                                    {"Delete"}
                                </button>
                            }
                        </div>
                    }
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
    let total_srd = order.total();
    let products_html = html! {
        {counted.iter().map(|(item, count)| {
            let price = item.price().parse::<f64>().unwrap_or(0.0);
            html! {
                    <div class="flex flex-row gap-2">
                        <p>{format!("{} x {}", count, item.name())}</p>
                        <p>{format!("SRD {:.2}", price * (*count as f64))}</p>
                    </div>
            }
        }).collect::<Html>()}
    };
    html! {
        <div class="flex flex-col gap-4">
            {products_html}
            <div class="flex flex-row justify-between">
                <p class="text-lg font-bold">{"Total"}</p>
                <p class="text-lg font-bold">{format!("SRD {:.2}", total_srd)}</p>
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ProductListItemProps {
    pub product: ProductItem,
    pub count: u32,
}
#[function_component(ProductListItem)]
pub fn product_list_item(props: &ProductListItemProps) -> Html {
    let ProductListItemProps { product, count } = props;
    html! {
        <div class="mt-5 space-y-3 flex items-center justify-between">
            <div class="flex items-center gap-5">
                <img src={product.thumbnail_url()} alt={product.name()} class="w-20 block object-contain" />
                <div>
                    <p class="text-gray-500 font-bold text-md">{product.name()}</p>
                    <p class="text-gray-500 font-light line-clamp-3">{product.details()}</p>
                    <p class="text-gray-500 font-bold text-md uppercase">{product.sku()}</p>
                </div>
            </div>

            <div class="flex flex-col items-center gap-2">
                <p class="text-gray-500 font-bold text-xl">{product.price()}</p>
                <p class="text-gray-500 font-bold text-md">{format!("x{}", count)}</p>
            </div>
        </div>
    }
}
