use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

use crate::Editor;

#[derive(Debug, Clone, PartialEq, Eq, Params)]
pub struct CatalogParams {
    search: Option<String>,
}

#[component]
pub fn Catalog() -> impl IntoView {
    let initial_search = use_params::<CatalogParams>()
        .get()
        .map(|p| p.search)
        .unwrap_or_default()
        .unwrap_or_default();
    let (search, _) = create_signal(initial_search);
    let (body, set_body) = create_signal(Vec::new());
    let items: Vec<(CatalogItem, _)> = CATALOG.with(|catalog| {
        catalog
            .iter()
            .map(|item| {
                (
                    item.clone(),
                    view!(<Editor example={&item.code} no_run=true />),
                )
            })
            .collect()
    });
    let update_search = move |search_text: &str| {
        let search_text = search_text.to_lowercase();
        set_body.set(
            items
                .iter()
                .filter(|(item, _)| {
                    search_text.is_empty()
                        || item.code.contains(&search_text)
                        || item.description.to_lowercase().contains(&search_text)
                })
                .map(|(item, view)| {
                    view! {
                        <div class="catalog-item">
                            <div style="width: 68%">{ view }</div>
                            <div style="width: 28%">{ &item.description }</div>
                        </div>
                    }
                })
                .collect(),
        );
    };
    update_search(&search.get());
    let on_search_input = move |event: Event| {
        let elem: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
        update_search(&elem.value());
    };
    view! {
        <h1>"Function Catalog"</h1>
        <p>"This is a currated list of Uiua functions for solving common problems."</p>
        <div class="input-div">
            <input
                type="text"
                placeholder="Search"
                value={ search.get() }
                on:input=on_search_input />
        </div>
        <br/>
        { body }
    }
}

#[derive(Clone)]
struct CatalogItem {
    code: String,
    description: String,
}

macro_rules! catalog {
    ($(#[doc = $desc:literal] $code:literal),* $(,)?) => {
        thread_local! {
            static CATALOG: Vec<CatalogItem> = vec![
                $(
                    CatalogItem {
                        code: $code.to_string(),
                        description: $desc.to_string(),
                    }
                ),*
            ];
        }
    };
}

catalog!(
    /// Reverse each row of an array
    "≡⇌ [1_2_3 4_5_6]",
    /// Get the sum of an array
    "/+ [1 2 3 4 5]",
    /// Create an identity matrix
    "⊞=.⇡ 5",
    /// Create an X matrix
    "↥⇌.⊞=.⇡ 5",
    /// Get the product of an array
    "/× [1 2 3 4 5]",
    /// Convert a number to a string
    "$\"_\" 17",
    /// Find the indices of all 1s
    "▽∶⇡⧻. [0 1 0 0 1]",
    /// Split an array by a delimiter
    "⊜□≠, @, \"split,this,up\"",
    /// Split an array by a delimiter with fill elements
    "⍛@ ⊜·≠, @, \"split,this,up\"",
    /// Find the nth fibonacci number
    ";⍥(+,∶)↶.1 10",
    /// Filter by a fixed predicate
    "▽ =0◿2 . ⇡10",
    /// Filter by a dynamic predicate
    "▽!∶, (=0◿2) ⇡10",
);
