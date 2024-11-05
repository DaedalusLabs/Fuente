use gloo::utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::Element;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    #[wasm_bindgen(js_name = agGrid)]
    pub type AgGrid;
    #[wasm_bindgen(static_method_of = AgGrid, js_name = createGrid, js_class = "agGrid")]
    pub fn create_grid(grid_element: &Element, options: JsValue) -> AgGrid;
    #[wasm_bindgen(method, js_name = "setGridOption")]
    pub fn set_grid_option(this: &AgGrid, option: &str, value: JsValue);
    #[wasm_bindgen(method, js_name = refreshCells)]
    pub fn refresh_cells(this: &AgGrid);
    #[wasm_bindgen(method, js_name = sizeColumnsToFit)]
    pub fn size_columns_to_fit(this: &AgGrid);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnDefinition {
    pub field: String,
    #[serde(rename = "headerName")]
    pub header_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resizable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cellRenderer")]
    pub cell_renderer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefaultColDef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resizable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgGridOptions<T>
where
    T: Serialize,
{
    #[serde(rename = "rowData")]
    pub row_data: Vec<T>,
    #[serde(rename = "columnDefs")]
    pub column_defs: Vec<ColumnDefinition>,
    #[serde(rename = "defaultColDef")]
    pub default_col_def: DefaultColDef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    #[serde(rename = "paginationPageSize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_page_size: Option<i32>,
    #[serde(rename = "rowSelection")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_selection: Option<String>,
}
impl<T> AgGridOptions<T>
where
    T: Serialize,
{
    pub fn new(row_data: Vec<T>) -> Self {
        AgGridOptions {
            row_data,
            column_defs: Vec::new(),
            default_col_def: DefaultColDef {
                sortable: Some(true),
                filter: Some(true),
                resizable: Some(true),
                flex: Some(1),
                min_width: Some(100),
            },
            pagination: Some(true),
            pagination_page_size: Some(10),
            row_selection: Some("single".to_string()),
        }
    }

    pub fn with_columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.column_defs = columns;
        self
    }

    pub fn with_default_col_def(mut self, default_col_def: DefaultColDef) -> Self {
        self.default_col_def = default_col_def;
        self
    }

    pub fn with_pagination(mut self, enabled: bool, page_size: Option<i32>) -> Self {
        self.pagination = Some(enabled);
        self.pagination_page_size = page_size;
        self
    }

    pub fn with_row_selection(mut self, selection_type: &str) -> Self {
        self.row_selection = Some(selection_type.to_string());
        self
    }
}

impl<T> From<AgGridOptions<T>> for JsValue
where
    T: Serialize,
{
    fn from(options: AgGridOptions<T>) -> JsValue {
        JsValue::from_serde(&options).unwrap()
    }
}

pub fn create_column(field: &str, header: &str) -> ColumnDefinition {
    ColumnDefinition {
        field: field.to_string(),
        header_name: header.to_string(),
        sortable: None,
        filter: None,
        resizable: None,
        width: None,
        pinned: None,
        cell_renderer: None,
    }
}
