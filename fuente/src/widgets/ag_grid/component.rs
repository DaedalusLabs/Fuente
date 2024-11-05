use super::{AgGrid, AgGridOptions, ColumnDefinition};
use serde::Serialize;
use web_sys::HtmlElement;
use yew::prelude::*;
use gloo::console; 

#[derive(Properties, Clone, PartialEq)]
pub struct AgGridProps<T>
where
    T: Serialize + Clone + PartialEq,
{
    pub data: Vec<T>,
    pub columns: Vec<ColumnDefinition>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub pagination: bool,
    #[prop_or(10)]
    pub page_size: i32,
    #[prop_or(false)]
    pub auto_size: bool,
    #[prop_or_default]
    pub on_row_selected: Option<Callback<T>>,
}

#[function_component(AgGridComponent)]
pub fn ag_grid_component<T>(props: &AgGridProps<T>) -> Html
where
    T: Serialize + Clone + PartialEq + 'static,
{
    let grid_ref = use_node_ref();
    let grid = use_state(|| None::<AgGrid>);

    // Initialize grid
    {
        let grid = grid.clone();
        let grid_ref = grid_ref.clone();
        let data = props.data.clone();
        let columns = props.columns.clone();
        let pagination = props.pagination;
        let page_size = props.page_size;
        let auto_size = props.auto_size;

        use_effect_with(
            (),
            move |_| {
                if let Some(element) = grid_ref.cast::<HtmlElement>() {
                    let options = AgGridOptions::new(data)
                        .with_columns(columns)
                        .with_pagination(pagination, Some(page_size))
                        .with_row_selection("single");

                    let ag_grid = AgGrid::create_grid(&element, options.into());
                    
                    if auto_size {
                        ag_grid.size_columns_to_fit();
                    }
                    
                    grid.set(Some(ag_grid));
                }
                || ()
            },
        );
    }

    // Update data when props change
    {
        let grid = grid.clone();
        let data = props.data.clone();
        use_effect_with(
            data.clone(),
            move |_| {
                if let Some(grid) = (*grid).clone() {
                    match serde_wasm_bindgen::to_value(&data) {
                        Ok(data_js) => {
                            grid.set_grid_option("rowData", data_js);
                            grid.refresh_cells();
                        },
                        Err(e) => {
                            console::error!("Failed to serialize grid data:", e.to_string());
                        }
                    }
                }
                || ()
            },
        );
    }

    html! {
        <div 
            ref={grid_ref} 
            class={classes!("ag-theme-alpine", props.class.clone())} 
            style="width: 100%; height: 500px;"
        />
    }
}