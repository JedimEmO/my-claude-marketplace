use dominator::{Dom, clone, events, html};
use futures_signals::signal::SignalExt;

use super::items::item_list;
use super::login::login_form;
use super::state::AppState;

pub fn app_root() -> Dom {
    let state = AppState::new();

    html!("div", {
        .style("min-height", "100vh")
        .style("color", "white")
        .child_signal(state.auth.token.signal_cloned().map(clone!(state => move |token| {
            if token.is_some() {
                Some(app_shell(state.clone()))
            } else {
                Some(login_form(state.clone()))
            }
        })))
    })
}

fn app_shell(state: AppState) -> Dom {
    html!("div", {
        .style("padding", "2rem")
        .children(&mut [
            // Header with logout
            html!("div", {
                .style("display", "flex")
                .style("justify-content", "space-between")
                .style("align-items", "center")
                .style("margin-bottom", "2rem")
                .children(&mut [
                    html!("h1", {
                        .style("font-size", "1.875rem")
                        .style("font-weight", "bold")
                        .text("Inventory")
                    }),
                    html!("div", {
                        .style("display", "flex")
                        .style("align-items", "center")
                        .style("gap", "1rem")
                        .children(&mut [
                            html!("span", {
                                .style("color", "#9ca3af")
                                .style("font-size", "0.875rem")
                                .text_signal(state.auth.user_id.signal_cloned().map(|u| {
                                    u.unwrap_or_default()
                                }))
                            }),
                            html!("button", {
                                .style("background", "#374151")
                                .style("color", "#d1d5db")
                                .style("padding", "0.375rem 0.75rem")
                                .style("border-radius", "0.25rem")
                                .style("border", "none")
                                .style("cursor", "pointer")
                                .style("font-size", "0.875rem")
                                .text("Logout")
                                .event(clone!(state => move |_: events::Click| {
                                    state.auth.logout();
                                }))
                            }),
                        ])
                    }),
                ])
            }),
            item_list(state),
        ])
    })
}
