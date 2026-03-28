use dominator::{Dom, clone, events, html};
use futures_signals::signal::Mutable;
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use wasm_bindgen_futures::spawn_local;

use app_core::dto::{CreateItemRequest, ItemListResponse, ItemResponse};

use super::http::{API_BASE, bind_input_value, delete_json, get_json, post_json};
use super::state::AppState;

pub fn item_list(state: AppState) -> Dom {
    let items: MutableVec<ItemResponse> = MutableVec::new();
    let name_input = Mutable::new(String::new());
    let quantity_input = Mutable::new(String::new());

    // Load items on mount
    {
        let items = items.clone();
        spawn_local(async move {
            if let Ok(response) =
                get_json::<ItemListResponse>(&format!("{API_BASE}/v1/items")).await
            {
                items.lock_mut().replace_cloned(response.items);
            }
        });
    }

    html!("div", {
        .style("display", "flex")
        .style("flex-direction", "column")
        .style("gap", "1.5rem")
        .children(&mut [
            // Create form
            html!("div", {
                .style("display", "flex")
                .style("gap", "1rem")
                .style("align-items", "flex-end")
                .children(&mut [
                    html!("div", {
                        .children(&mut [
                            html!("label", {
                                .style("display", "block")
                                .style("font-size", "0.875rem")
                                .style("color", "#9ca3af")
                                .style("margin-bottom", "0.25rem")
                                .text("Name")
                            }),
                            html!("input" => web_sys::HtmlInputElement, {
                                .style("background", "#1f2937")
                                .style("border", "1px solid #374151")
                                .style("border-radius", "0.25rem")
                                .style("padding", "0.5rem 0.75rem")
                                .style("color", "white")
                                .attr("type", "text")
                                .attr("placeholder", "Item name")
                                .event(bind_input_value(name_input.clone()))
                            }),
                        ])
                    }),
                    html!("div", {
                        .children(&mut [
                            html!("label", {
                                .style("display", "block")
                                .style("font-size", "0.875rem")
                                .style("color", "#9ca3af")
                                .style("margin-bottom", "0.25rem")
                                .text("Quantity")
                            }),
                            html!("input" => web_sys::HtmlInputElement, {
                                .style("background", "#1f2937")
                                .style("border", "1px solid #374151")
                                .style("border-radius", "0.25rem")
                                .style("padding", "0.5rem 0.75rem")
                                .style("color", "white")
                                .style("width", "6rem")
                                .attr("type", "number")
                                .attr("placeholder", "0")
                                .event(bind_input_value(quantity_input.clone()))
                            }),
                        ])
                    }),
                    html!("button", {
                        .style("background", "#2563eb")
                        .style("color", "white")
                        .style("padding", "0.5rem 1rem")
                        .style("border-radius", "0.25rem")
                        .style("border", "none")
                        .style("cursor", "pointer")
                        .text("Add Item")
                        .event(clone!(state, items, name_input, quantity_input => move |_: events::Click| {
                            let name = name_input.get_cloned();
                            let quantity: u32 = quantity_input.get_cloned().parse().unwrap_or(0);
                            if name.is_empty() || quantity == 0 {
                                return;
                            }
                            let token = state.auth.token.get_cloned().unwrap_or_default();
                            let items = items.clone();
                            spawn_local(async move {
                                let req = CreateItemRequest { name, quantity };
                                if let Ok(item) = post_json::<_, ItemResponse>(
                                    &format!("{API_BASE}/v1/items"),
                                    &req,
                                    Some(&token),
                                ).await {
                                    items.lock_mut().push_cloned(item);
                                }
                            });
                        }))
                    }),
                ])
            }),
            // Item list
            html!("div", {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "0.5rem")
                .children_signal_vec(items.signal_vec_cloned().map(clone!(state, items => move |item| {
                    let id = item.id.to_string();
                    html!("div", {
                        .style("display", "flex")
                        .style("justify-content", "space-between")
                        .style("align-items", "center")
                        .style("background", "#1f2937")
                        .style("border-radius", "0.25rem")
                        .style("padding", "1rem")
                        .children(&mut [
                            html!("div", {
                                .children(&mut [
                                    html!("span", {
                                        .style("font-weight", "500")
                                        .text(&item.name)
                                    }),
                                    html!("span", {
                                        .style("color", "#9ca3af")
                                        .style("margin-left", "1rem")
                                        .text(&format!("qty: {}", item.quantity))
                                    }),
                                ])
                            }),
                            html!("button", {
                                .style("color", "#f87171")
                                .style("background", "none")
                                .style("border", "none")
                                .style("cursor", "pointer")
                                .text("Delete")
                                .event(clone!(state, items, id => move |_: events::Click| {
                                    let token = state.auth.token.get_cloned().unwrap_or_default();
                                    let items = items.clone();
                                    let id = id.clone();
                                    spawn_local(async move {
                                        if delete_json(&format!("{API_BASE}/v1/items/{id}"), &token).await.is_ok() {
                                            let mut lock = items.lock_mut();
                                            if let Some(pos) = lock.iter().position(|i| i.id.to_string() == id) {
                                                lock.remove(pos);
                                            }
                                        }
                                    });
                                }))
                            }),
                        ])
                    })
                })))
            }),
        ])
    })
}
