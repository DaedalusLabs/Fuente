use crate::contexts::{CommerceDataAction, CommerceDataStore};
use fuente::{
    mass::{
        CardComponent, DrawerSection, LoadingScreen, MoneyInput, ProductMenuCard, SimpleFormButton,
        SimpleInput, SimpleSelect, SimpleTextArea,
    },
    models::{ProductCategory, ProductItem, ProductMenu, ProductMenuIdb},
};

use minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

#[function_component(ProductsPage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>();
    if commerce_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    let menu_ctx = commerce_ctx.unwrap().menu();
    let edit_screen_state = use_state(|| false);
    let open_screen_clone = edit_screen_state.clone();
    let open_edit_screen = Callback::from(move |_| open_screen_clone.set(true));
    match *edit_screen_state {
        true => html! {<NewProductListSection />},
        false => html! {
            <div class="flex flex-col flex-1 gap-8 pr-4">
                <div class="flex flex-row w-full justify-between items-center">
                    <h2 class="text-4xl">{"My Products"}</h2>
                    <button
                        onclick={open_edit_screen}
                        class="w-fit h-fit bg-fuente-light text-white
                        font-mplus px-4 py-2 rounded-3xl"
                        >{"Edit Menu"}</button>
                </div>
                {if let Some(menu) = menu_ctx {
                    html!{<ProductMenuCard menu={menu} />}
                } else {
                    html! {}
                }}
            </div>
        },
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct EditMenuProps {
    pub screen_handle: UseStateHandle<bool>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct NewMenuProps {
    pub menu: UseStateHandle<Option<ProductMenu>>,
}

#[function_component(NewProductListSection)]
pub fn product_list_section() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce data context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Consumer context not found");
    let menu_state = use_state(|| commerce_ctx.menu());

    let new_menu = menu_state.clone();
    let handle = commerce_ctx.clone();
    let keys = key_ctx.get_nostr_key();
    let sender = relay_ctx.send_note.clone();
    let onclick = Callback::from(move |_: MouseEvent| {
        if let (Some(new_menu), Some(key)) = ((*new_menu).clone(), keys.clone()) {
            let db_entry = ProductMenuIdb::new(new_menu, &key);
            sender.emit(db_entry.note());
            handle.dispatch(CommerceDataAction::UpdateProductList(db_entry))
        }
    });
    html! {
        <div
            class="flex flex-col gap-4">
            <div class="w-full flex flex-row justify-between items-center">
                <button {onclick}
                    class="w-fit h-fit bg-fuente-light text-white text-sm
                    font-mplus px-4 py-2 rounded-3xl"
                    >{"Save Menu"}</button>
            </div>
            <AddCategoryForm menu={menu_state.clone()} />
            <AddProductForm menu={menu_state.clone()} />
            {if let Some(menu) = menu_state.as_ref() {
                html!{<ProductMenuCard menu={menu.clone()} />}
            } else {
                html! {}
            }}
        </div>
    }
}

#[function_component(AddCategoryForm)]
pub fn add_category_form(props: &NewMenuProps) -> Html {
    let close_handle = use_state(|| false);
    let menu = props.menu.clone();
    let handle = menu.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let category_name = form
            .input_value("category_name")
            .expect("Failed to get category name");
        match (*menu).clone() {
            Some(mut menu) => {
                let new_category = ProductCategory::new(menu.categories().len(), category_name);
                menu.update_category_name(new_category);
                handle.set(Some(menu));
            }
            None => {
                let mut new_menu = ProductMenu::new();
                let new_category = ProductCategory::new(0, category_name);
                new_menu.add_category(new_category);
                menu.set(Some(new_menu));
            }
        }
    });
    html! {
        <DrawerSection title={"Add Category"} open={close_handle.clone()}>
            <form
                class="flex flex-col gap-2 items-center p-4"
                {onsubmit}>
                <SimpleInput label="Categoria" name="category_name" value="" input_type="text" id="category_name" required={true} />
                <SimpleFormButton>{"Add Category"}</SimpleFormButton>
            </form>
        </DrawerSection>
    }
}

#[function_component(AddProductForm)]
pub fn add_product_form(props: &NewMenuProps) -> Html {
    let close_handle = use_state(|| false);
    let NewMenuProps { menu } = props;
    let handle = menu.clone();
    let menu_copy = menu.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let product_category = form
            .select_value("product_category")
            .expect("Failed to get category");
        let product_name = form
            .input_value("product_name")
            .expect("Failed to get name");
        let product_price = form
            .input_value("product_price")
            .expect("Failed to get price");
        let description = form
            .textarea_value("description")
            .expect("Failed to get description");
        let menu = menu_copy.clone();
        match (*menu).clone() {
            Some(mut menu) => {
                let categories = menu.categories();
                let category = categories
                    .iter()
                    .find(|category| category.name() == product_category)
                    .expect("Category not found");
                let product = ProductItem::new(
                    category.products().len(),
                    product_name,
                    product_price,
                    description,
                    category.id(),
                );
                menu.add_product(category.id(), product);
                handle.set(Some(menu));
            }
            None => {}
        }
    });
    html! {
        <DrawerSection title={"Add Product"} open={close_handle.clone()}>
            <CardComponent>
                <form
                    class="flex flex-col gap-2 items-center"
                    {onsubmit}>
                    <SimpleSelect label="Categoria" name="product_category" id="product_category">
                        {if let Some(menu) = (*props.menu).clone() {
                            menu.categories().iter().map(|category| {
                                html! {
                                    <option value={category.name()}>{category.name()}</option>
                                }
                            }).collect::<Html>()
                        } else {
                            html! {}
                        }}
                    </SimpleSelect>
                    <SimpleInput label="Product" name="product_name" value="" input_type="text" id="product_name" required={true} />
                    <MoneyInput label="Price" name="product_price" value="" id="product_price" required={true} input_type="number" />
                    <SimpleTextArea label="Description" name="description" value="" input_type="text" id="description" required={true} />
                    <SimpleFormButton>{"Add Product"}</SimpleFormButton>
                </form>
            </CardComponent>
        </DrawerSection>
    }
}

#[function_component(AddProductExtraForm)]
pub fn add_product_extra_form(props: &NewMenuProps) -> Html {
    let _close_handle = use_state(|| false);
    let NewMenuProps { menu } = props;
    let handle = menu.clone();
    let menu_copy = menu.clone();
    let _onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let product_category = form
            .select_value("product_category")
            .expect("Failed to get category");
        let product_name = form
            .select_value("product_name")
            .expect("Failed to get name");
        let product_price = form
            .input_value("product_price")
            .expect("Failed to get price");
        let description = form
            .textarea_value("description")
            .expect("Failed to get description");
        let menu = menu_copy.clone();
        match (*menu).clone() {
            Some(mut menu) => {
                let categories = menu.categories();
                let category = categories
                    .iter()
                    .find(|category| category.name() == product_category)
                    .expect("Category not found");
                let product = ProductItem::new(
                    category.products().len(),
                    product_name,
                    product_price,
                    description,
                    category.id(),
                );
                menu.add_product(category.id(), product);
                handle.set(Some(menu));
            }
            None => {}
        }
    });

    html! {}
}
