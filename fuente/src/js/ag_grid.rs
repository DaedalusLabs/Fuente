use gloo::utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
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
    #[wasm_bindgen(method, js_name = setGridOption)]
    pub fn set_grid_options(this: &AgGrid, option: &str, row_data: JsValue);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumunDefinitions {
    field: &'static str,
    #[serde(rename = "headerName")]
    header_name: &'static str,
}
#[derive(Debug, Clone, Serialize)]
pub struct AgGridOptions<T>
where
    T: Serialize,
{
    #[serde(rename = "rowData")]
    row_data: Vec<T>,
    #[serde(rename = "columnDefs")]
    column_defs: Vec<ColumunDefinitions>,
    #[serde(rename = "defaultColDef")]
    default_col_def: Value,
}
impl<T> AgGridOptions<T>
where
    T: Serialize,
{
    pub fn doctor_profiles_grid(row_data: Vec<T>) -> Self {
        let column_defs = vec![
            ColumunDefinitions {
                field: "name",
                header_name: "Nombre",
            },
            ColumunDefinitions {
                field: "email",
                header_name: "Email",
            },
            ColumunDefinitions {
                field: "phone",
                header_name: "Teléfono",
            },
            ColumunDefinitions {
                field: "especialidad",
                header_name: "Especialidad",
            },
            ColumunDefinitions {
                field: "jvpm",
                header_name: "No. Junta",
            },
            ColumunDefinitions {
                field: "dui",
                header_name: "DUI",
            },
            ColumunDefinitions {
                field: "pubkey",
                header_name: "CryptoID",
            },
        ];
        let default_col_def = json!({
            "flex": 1,
        });
        AgGridOptions {
            row_data,
            column_defs,
            default_col_def,
        }
    }
}
impl<T> Into<JsValue> for AgGridOptions<T> 
where
    T: Serialize,
{
    fn into(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}
