use crate::contexts::{CommerceDataAction, CommerceDataStore};
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{
        templates::SettingsPageTemplate, CardComponent, DrawerSection, LoadingScreen, MoneyInput,
        SimpleFormButton, SimpleInput, SimpleTextArea,
    },
    models::{ProductCategory, ProductItem, ProductMenu, ProductMenuIdb},
};

use fuente::mass::{ImageUploadInput, ProductCard};
use lucide_yew::{SquarePen, Trash};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ProductPageSection {
    Products,
    OnSale,
    Banner,
}

#[function_component(ProductsPage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>();
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("No LanguageConfigsStore found");
    let translations = language_ctx.translations();
    let current_page = use_state(|| ProductPageSection::Products);
    let add_product_modal = use_state(|| false);
    let add_category_modal = use_state(|| false);
    if commerce_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    let go_to_products = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(ProductPageSection::Products))
    };
    let go_to_on_sale = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(ProductPageSection::OnSale))
    };
    let go_to_banner = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(ProductPageSection::Banner))
    };
    let onclick_new_product = {
        let modal = add_product_modal.clone();
        Callback::from(move |_| {
            modal.set(true);
        })
    };
    let new_product_button = {
        html! {
            <div class="flex items-center gap-4">
                <div class="flex justify-center items-center">
                    <button
                        type="button" onclick={onclick_new_product}
                        class="flex items-center bg-fuente-buttons px-7 py-4 rounded-full text-fuente-forms space-x-2 font-bold">
                        {&translations["admin_store_add_product_button"]}
                    </button>
                </div>
            </div>
        }
    };
    let onclick_new_category = {
        let modal = add_category_modal.clone();
        Callback::from(move |_| {
            modal.set(true);
        })
    };
    let new_product_category = {
        html! {
            <div class="flex items-center gap-4">
                <div class="flex justify-center items-center">
                    <button
                        type="button" onclick={onclick_new_category}
                        class="flex items-center bg-fuente-buttons px-7 py-4 rounded-full text-fuente-forms space-x-2 font-bold">
                        {&translations["store_products_form_label_add_category"]}
                    </button>
                </div>
            </div>
        }
    };
    html! {
        <>
        <SettingsPageTemplate
            heading={translations["admin_store_new_products_heading"].clone()}
            options={ vec![
                new_product_category,
                new_product_button,
            ]}
            sidebar_options={ vec![
                (translations["admin_store_new_products_button"].clone(), go_to_products, if *current_page == ProductPageSection::Products { true } else { false }),
                (translations["admin_store_sale_products_button"].clone(), go_to_on_sale, if *current_page == ProductPageSection::OnSale { true } else { false }),
                (translations["admin_store_banner_button"].clone(), go_to_banner, if *current_page == ProductPageSection::Banner { true } else { false }),
            ]}
            content_button={None} >
            <>
                {match *current_page {
                    ProductPageSection::Products => {
                        html! {
                            <AllProductsSection />
                        }
                    }
                    ProductPageSection::OnSale => {
                        html! {
                        }
                    }
                    ProductPageSection::Banner => {
                        html! {
                        }
                    }
                }}
            </>
        </SettingsPageTemplate>
        <AddProductModal screen_handle={add_product_modal.clone()} />
        <AddCategoryModal screen_handle={add_category_modal.clone()} />
        </>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct EditMenuProps {
    pub screen_handle: UseStateHandle<bool>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct EditProductFormProps {
    pub menu: UseStateHandle<Option<ProductMenu>>,
    pub product: ProductItem,
    pub on_cancel: Callback<MouseEvent>,
}

#[function_component(AddCategoryForm)]
pub fn add_category_form() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrIdStore found");
    let relay_ctx = use_context::<NostrProps>().expect("No RelayProps found");
    let menu = commerce_ctx.menu();
    let onsubmit = {
        let sender = relay_ctx.send_note.clone();
        let key = key_ctx.get_nostr_key().expect("No user keys found");
        let handle = commerce_ctx.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let form = HtmlForm::new(e).expect("Failed to get form");
            let category_name = form
                .input_value("category_name")
                .expect("Failed to get category name");
            match (menu).clone() {
                Some(mut menu) => {
                    let new_category = ProductCategory::new(menu.categories().len(), category_name);
                    menu.update_category_name(new_category);
                    let db_entry = ProductMenuIdb::new(menu, &key);
                    sender.emit(db_entry.note());
                    handle.dispatch(CommerceDataAction::UpdateProductList(db_entry))
                }
                None => {
                    let mut new_menu = ProductMenu::new();
                    let new_category = ProductCategory::new(0, category_name);
                    new_menu.add_category(new_category);
                    let db_entry = ProductMenuIdb::new(new_menu, &key);
                    sender.emit(db_entry.note());
                    handle.dispatch(CommerceDataAction::UpdateProductList(db_entry))
                }
            }
        })
    };
    html! {
        <main class="bg-white rounded-2xl p-10 max-w-6xl mx-auto flex-1">
            <form
                class="flex flex-col gap-2 items-center p-4"
                {onsubmit}>
                <SimpleInput label="Categoria" name="category_name" value="" input_type="text" id="category_name" required={true} />
                <SimpleFormButton>{"Add Category"}</SimpleFormButton>
            </form>
        </main>
    }
}

#[function_component(AddProductModal)]
pub fn add_product_modal(props: &EditMenuProps) -> Html {
    let close_handle = props.screen_handle.clone();
    match *close_handle {
        true => {
            html! {
                <>
                    <div onclick={Callback::from(move |_| close_handle.set(false))}
                        class="fixed inset-0 flex justify-center items-center bg-neutral-500 opacity-80 z-20">
                    </div>
                    <div class="fixed top-8 z-30 h-fit w-fit left-12">
                         <AddProductForm />
                    </div>
                </>
            }
        }
        false => {
            html! {}
        }
    }
}

#[function_component(AddCategoryModal)]
pub fn add_product_modal(props: &EditMenuProps) -> Html {
    let close_handle = props.screen_handle.clone();
    match *close_handle {
        true => {
            html! {
                <>
                    <div onclick={Callback::from(move |_| close_handle.set(false))}
                        class="fixed inset-0 flex justify-center items-center bg-neutral-500 opacity-80 z-20">
                    </div>
                    <div class="fixed top-8 z-30 h-fit w-fit left-12">
                        <AddCategoryForm />
                    </div>
                </>
            }
        }
        false => {
            html! {}
        }
    }
}

#[function_component(AddProductForm)]
pub fn add_product_form() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    let menu = commerce_ctx.menu();

    let image_url = use_state(|| None::<String>);
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrIdStore found");
    let relay_ctx = use_context::<NostrProps>().expect("No RelayProps found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No LanguageStore found");
    let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
    let thumbnail_url = use_state(|| None::<String>);
    let discount_enabled = use_state(|| false);
    let translations = language_ctx.translations();

    let onsubmit = {
        let image_url = image_url.clone();
        let thumbnail_url = thumbnail_url.clone();
        let discount_enabled = discount_enabled.clone();
        let sender = relay_ctx.send_note.clone();
        let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
        let handle = commerce_ctx.clone();
        let menu = menu.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            gloo::console::log!("Form submitting with:");
            gloo::console::log!("Image URL:", format!("{:?}", *image_url));
            gloo::console::log!("Thumbnail URL:", format!("{:?}", *thumbnail_url));

            let form = HtmlForm::new(e).expect("Failed to get form element");
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
            let details = form
                .textarea_value("details")
                .expect("Failed to get details");
            let discount = form
                .input_value("discount")
                .ok()
                .filter(|_| *discount_enabled);

            match (menu).clone() {
                Some(mut menu) => {
                    let categories = menu.categories();
                    let category = categories
                        .iter()
                        .find(|category| category.name() == product_category)
                        .expect("Category not found");

                    let mut product = ProductItem::new(
                        category.products().len(),
                        product_name,
                        product_price,
                        description,
                        category.id(),
                    );

                    // Set image URL if available, log the process
                    if let Some(url) = (*image_url).clone() {
                        gloo::console::log!("Setting main image URL:", url.clone());
                        product.set_image_url(url);
                    }
                    // Set thumbnail
                    if let Some(url) = (*thumbnail_url).clone() {
                        gloo::console::log!("Setting thumbnail URL:", url.clone());
                        product.set_thumbnail_url(url);
                    } else {
                        gloo::console::warn!("No image URL found for product");
                    }
                    product.set_details(details);
                    product.set_discount(discount);
                    gloo::console::log!("Final product:", format!("{:?}", product));

                    menu.add_product(category.id(), product);
                    let db_entry = ProductMenuIdb::new(menu, &nostr_keys);
                    sender.emit(db_entry.note());
                    handle.dispatch(CommerceDataAction::UpdateProductList(db_entry))
                }
                None => {}
            }
        })
    };

    let onchange_price = {
        Callback::from(move |_e: Event| {
            let document =
                nostr_minions::browser_api::HtmlDocument::new().expect("Failed to get document");
            let price = document
                .find_element_by_id::<HtmlInputElement>("price")
                .expect("Failed to parse price")
                .value_as_number();
            let discount = document
                .find_element_by_id::<HtmlInputElement>("discount")
                .expect("Failed to parse discount")
                .value_as_number();
            let total_price = document
                .find_element_by_id::<HtmlInputElement>("product_price")
                .expect("Failed to parse total price");
            if discount > 0.0 {
                total_price.set_value(&format!("{:.2}", price - discount));
            } else {
                total_price.set_value(&format!("{:.2}", price));
            }
        })
    };

    html! {
        <main class="bg-white rounded-2xl p-10 max-w-6xl mx-auto flex-1">
            <form {onsubmit} class="grid grid-cols-2 gap-20">
                <div>
                    <div class="space-y-2">
                        <label for="product_name" class="text-gray-400 font-light block text-md">{&translations["store_products_form_label_name"]}</label>
                        <input type="text" id="product_name" class="border border-fuente rounded-xl p-2 w-full" required={true} />
                    </div>

                    <p class="text-gray-400 font-light mt-5 text-sm">{&translations["store_products_form_label_information"]}</p>

                    <div class="mt-5 space-y-4">
                        <div class="grid grid-cols-2 items-center gap-4">
                            <label for="product_category" class="text-gray-400 font-semibold">{&translations["store_products_form_label_category"]}</label>
                            <select id="product_category" class="border border-fuente rounded-xl p-2 w-full" required={true}>
                                {if let Some(menu) = (menu).clone() {
                                    menu.categories().iter().map(|category| {
                                        html! {
                                            <option value={category.name()}>{category.name()}</option>
                                        }
                                    }).collect::<Html>()
                                    } else {
                                        html! {}
                                    }
                                }
                            </select>
                        </div>

                        <div class="space-y-2">
                            <label for="description" class="text-gray-400 font-light block">{&translations["store_products_form_label_details"]}</label>
                            <textarea id="details" class="border border-fuente rounded-xl p-2 w-full min-h-32"></textarea>
                        </div>

                        <div class="space-y-2">
                            <label for="description" class="text-gray-400 font-light block">{&translations["store_products_form_label_description"]}</label>
                            <textarea id="description" class="border border-fuente rounded-xl p-2 w-full min-h-32"></textarea>
                        </div>
                    </div>
                </div>

                <div>
                    <p class="text-gray-400 text-sm font-light">{&translations["store_products_form_label_photos"]}</p>
                    <div class="grid grid-cols-2 mt-2 gap-5">
                        // Large Image Upload
                        <div class="w-full flex flex-col gap-2">
                            <label class="text-xs font-bold text-neutral-400">
                                {"Product Image (Large)"}
                            </label>
                            <ImageUploadInput
                                url_handle={image_url.clone()}
                                nostr_keys={nostr_keys.clone()}
                                classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")}
                                input_id="large-image-upload"  // Unique ID for large image
                            />
                            // can be removed
                            {if let Some(_url) = (*image_url).clone() {
                                html! {
                                    <span class="text-xs text-green-500 mt-1">{"✓ Large image uploaded"}</span>
                                }
                            } else {
                                html! {}
                            }}
                        </div>

                        // Thumbnail Image Upload
                        <div class="w-full flex flex-col gap-2">
                            <label class="text-xs font-bold text-neutral-400">
                                {"Product Thumbnail"}
                            </label>
                            <ImageUploadInput
                                url_handle={thumbnail_url.clone()}
                                nostr_keys={nostr_keys}
                                classes={classes!("min-w-16", "min-h-16", "h-16", "w-16")}
                                input_id="thumbnail-image-upload"  // Unique ID for thumbnail image
                            />
                        </div>
                    </div>

                    <div class="mt-10">
                        <p class="text-gray-400 font-light text-md">{&translations["store_products_form_label_price_info"]}</p>
                        <div class="mt-5 space-y-5">
                            <div class="w-full flex justify-between">
                                <label for="price" class="text-gray-400 font-semibold ">{&translations["store_products_form_label_original_price"]}</label>
                                <input onchange={onchange_price.clone()} step={"0.01"} type="number" id="price" class="border border-fuente rounded-xl p-2 max-w-24" />
                            </div>

                            <div class="w-full flex justify-between">
                                <label for="discount" class="text-gray-400 font-semibold">{&translations["store_products_form_label_discount"]}</label>
                                <input onchange={onchange_price} step={"0.01"} type="number" id="discount" class="border border-fuente rounded-xl p-2 max-w-24" />
                            </div>

                            <div class="w-full flex justify-between">
                                <label for="product_price" class="text-gray-400 font-semibold ">{&translations["store_products_form_label_total_price"]}</label>
                                <input step={"0.01"} type="number" id="product_price"  disabled={true}
                                    class="border border-fuente rounded-xl p-2 max-w-24" required={true} />
                            </div>
                        </div>

                    <button
                        type="submit"
                        class="bg-fuente-orange text-white font-semibold rounded-full py-3 w-full mt-10 text-center">
                        {&translations["store_products_form_label_add_button"]}
                    </button>
                </div>
            </div>
        </form>
    </main>
        }
}

#[function_component(AllProductsSection)]
pub fn product_list_section() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce data context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Consumer context not found");
    let menu_state = use_state(|| commerce_ctx.menu());
    let editing_product = use_state(|| None::<ProductItem>); // Editing product state

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
        <table class="table-auto w-full border-collapse">
            <thead>
                <tr>
                    <th></th>
                    <th class=" py-3 text-left text-md leading-4 font-semibold text-fuente text-lg" >{"Product Details"}</th>
                    <th class=" py-3 text-center text-md leading-4 font-semibold text-fuente text-lg">{"Price"}</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
                {
                    if let Some(menu) = menu_state.as_ref() {
                    menu.categories().iter().map(|category| {
                        {category.products().iter().map(|product| {
                            let menu_handle = menu_state.clone();

                            // Delete handler
                            let on_delete = {
                                let product_id = product.id();
                                let category_id = category.id();
                                let menu_handle = menu_handle.clone();
                                let handle = commerce_ctx.clone();  // Get commerce context
                                let keys = key_ctx.get_nostr_key().expect("No user keys found");
                                let sender = relay_ctx.send_note.clone();

                                Callback::from(move |_: MouseEvent| {
                                    if let Some(mut menu) = (*menu_handle).clone() {
                                        menu.remove_product(&category_id, &product_id);

                                        // Create ProductMenuIdb and broadcast changes
                                        let db_entry = ProductMenuIdb::new(menu.clone(), &keys);
                                        sender.emit(db_entry.note());
                                        handle.dispatch(CommerceDataAction::UpdateProductList(db_entry.clone()));

                                        // Update local state
                                        menu_handle.set(Some(menu));
                                    }
                                })
                            };

                            let on_edit = {
                                let editing_product = editing_product.clone();
                                let product_clone = product.clone();

                                Callback::from(move |_: MouseEvent| {
                                    editing_product.set(Some(product_clone.clone()));
                                })
                            };

                            html! {
                                <tr>
                                    <td class="py-8 pt-4 whitespace-nowrap">
                                        <img src={product.thumbnail_url()} alt="Product Image" class="w-32 bg-gray-200 rounded-2xl" />
                                    </td>
                                    <td class="px-6 py-8 whitespace-nowrap max-w-48">
                                        <p class="font-bold text-gray-500 mt-8">{product.name()}</p>
                                        <p class="font-thin text-gray-500 mt-3 text-wrap line-clamp-4">{product.details()}</p>
                                        <p class="font-bold text-gray-500">{product.sku()}</p>
                                    </td>
                                    <td class="px-6 py-8 whitespace-nowrap text-right text-4xl text-fuente font-semibold">
                                        <p class="mt-20">{product.price()}</p>
                                    </td>
                                    <td class="whitespace-nowrap text-center py-8 px-6 flex gap-2 items-center">
                                        <button onclick={on_edit} class="w-10 h-10 mt-20 text-fuente">
                                            <SquarePen class="cursor-pointer" />
                                        </button>
                                        <button onclick={on_delete} class="w-10 h-10 mt-20 text-red-600">
                                            <Trash class="cursor-pointer" />
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Html>()}
                    }).collect::<Html>()}
                    else {
                        html! {}
                    }
                }

            </tbody>
        </table>

    }
}

#[function_component(EditProductForm)]
pub fn edit_product_form(props: &EditProductFormProps) -> Html {
    let close_handle = use_state(|| false);
    let EditProductFormProps {
        menu: menu_handle,
        product,
        on_cancel,
    } = props.clone();

    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");

    let image_url = use_state(|| Some(product.image_url()));
    let thumbnail_url = use_state(|| Some(product.thumbnail_url()));
    let discount_enabled = use_state(|| product.discount().is_some());
    let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");

    let onsubmit = {
        let handle = commerce_ctx.clone();
        let sender = relay_ctx.send_note.clone();
        let on_cancel = on_cancel.clone();
        let thumbnail_url = thumbnail_url.clone();
        let discount_enabled = discount_enabled.clone();
        let image_url = image_url.clone();
        let nostr_keys = nostr_keys.clone();
        let product = product.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let form = HtmlForm::new(e).expect("Failed to get form");

            if let Some(mut menu) = (*menu_handle).clone() {
                let mut updated_product = product.clone();

                updated_product.set_name(form.input_value("product_name").expect("No name"));
                updated_product.set_price(form.input_value("product_price").expect("No price"));
                updated_product.set_details(form.textarea_value("details").expect("No details"));
                updated_product
                    .set_description(form.textarea_value("description").expect("No description"));
                updated_product.set_discount(if *discount_enabled {
                    form.input_value("discount").ok()
                } else {
                    None
                });

                if let Some(url) = (*image_url).clone() {
                    updated_product.set_image_url(url);
                }
                if let Some(url) = (*thumbnail_url).clone() {
                    updated_product.set_thumbnail_url(url);
                }

                // Remove old product first
                menu.remove_product(&updated_product.category_id(), &updated_product.id());

                menu.add_product(updated_product.category_id(), updated_product);

                let db_entry = ProductMenuIdb::new(menu.clone(), &nostr_keys);
                sender.emit(db_entry.note());
                handle.dispatch(CommerceDataAction::UpdateProductList(db_entry));
                menu_handle.set(Some(menu.clone()));
                on_cancel.emit(MouseEvent::new("click").unwrap());
            }
        })
    };

    html! {
        <DrawerSection title={"Edit Product"} open={close_handle.clone()}>
            <CardComponent>
                <form {onsubmit} class="flex flex-col gap-2 items-center">
                    <SimpleInput
                        label="Name"
                        value={product.name()}
                        id="product_name"
                        name="product_name"
                        input_type="text"
                        required={true}
                    />
                    <MoneyInput
                        label="Price"
                        value={product.price()}
                        id="product_price"
                        name="product_price"
                        input_type="number"
                        required={true}
                    />
                    <div class="grid grid-cols-2 gap-4 w-full">
                        <div class="w-full flex flex-col gap-2">
                            <label class="text-xs font-bold text-neutral-400">
                                {"Product Image"}
                            </label>
                            <ImageUploadInput
                                url_handle={image_url.clone()}
                                nostr_keys={nostr_keys.clone()}
                                classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")}
                                input_id="edit-large-image-upload"  // Unique ID for edit form large image
                            />
                        </div>
                        <div class="w-full flex flex-col gap-2">
                            <label class="text-xs font-bold text-neutral-400">
                                {"Thumbnail"}
                            </label>
                            <ImageUploadInput
                                url_handle={thumbnail_url.clone()}
                                nostr_keys={nostr_keys}
                                classes={classes!("min-w-16", "min-h-16", "h-16", "w-16")}
                                input_id="edit-thumbnail-upload"  // Unique ID for edit form thumbnail
                            />
                        </div>
                    </div>
                    <SimpleTextArea
                        label="Description"
                        value={product.description()}
                        id="description"
                        name="description"
                        input_type="text"
                        required={true}
                    />
                    <SimpleTextArea
                        label="Details"
                        value={product.details()}
                        id="details"
                        name="details"
                        input_type="text"
                        required={true}
                    />
                    <div class="w-full flex items-center gap-2">
                        <input
                            type="checkbox"
                            id="enable_discount"
                            checked={*discount_enabled}
                            onclick={{
                                let discount_enabled = discount_enabled.clone();
                                Callback::from(move |_| {
                                    discount_enabled.set(!*discount_enabled);
                                })
                            }}
                        />
                        <label for="enable_discount">{"Enable Discount"}</label>
                    </div>
                    {if *discount_enabled {
                        html! {
                            <MoneyInput
                                label="Discount Amount"
                                value={product.discount().unwrap_or_default()}
                                id="discount"
                                name="discount"
                                input_type="number"
                                required={true}
                            />
                        }
                    } else {
                        html! {}
                    }}
                    <div class="flex gap-2">
                        <button
                            type="submit"
                            class="bg-fuente text-white rounded-lg px-4 py-2"
                        >
                            {"Save Changes"}
                        </button>
                        <button
                            type="button"
                            onclick={on_cancel}
                            class="border border-gray-300 rounded-lg px-4 py-2"
                        >
                            {"Cancel"}
                        </button>
                    </div>
                </form>
            </CardComponent>
        </DrawerSection>
    }
}
