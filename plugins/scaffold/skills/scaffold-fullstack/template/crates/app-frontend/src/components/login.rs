use dominator::{Dom, clone, events, html};
use futures_signals::signal::{Mutable, SignalExt};
use wasm_bindgen_futures::spawn_local;

use app_core::dto::{LoginRequest, LoginResponse};

use super::http::{API_BASE, bind_input_value, post_json};
use super::state::AppState;

pub fn login_form(state: AppState) -> Dom {
    let username = Mutable::new(String::new());
    let password = Mutable::new(String::new());
    let error_msg = Mutable::new(Option::<String>::None);

    html!("div", {
        .style("display", "flex")
        .style("justify-content", "center")
        .style("align-items", "center")
        .style("min-height", "100vh")
        .children(&mut [
            html!("div", {
                .style("background", "#1f2937")
                .style("border-radius", "0.5rem")
                .style("padding", "2rem")
                .style("width", "20rem")
                .children(&mut [
                    html!("h2", {
                        .style("font-size", "1.5rem")
                        .style("font-weight", "bold")
                        .style("margin-bottom", "1.5rem")
                        .style("text-align", "center")
                        .text("Login")
                    }),
                    html!("div", {
                        .style_signal("display", error_msg.signal_cloned().map(|e| {
                            if e.is_some() { "block" } else { "none" }
                        }))
                        .style("background", "#7f1d1d")
                        .style("color", "#fca5a5")
                        .style("padding", "0.5rem 1rem")
                        .style("border-radius", "0.25rem")
                        .style("margin-bottom", "1rem")
                        .style("font-size", "0.875rem")
                        .text_signal(error_msg.signal_cloned().map(|e| e.unwrap_or_default()))
                    }),
                    html!("div", {
                        .style("margin-bottom", "1rem")
                        .children(&mut [
                            html!("label", {
                                .style("display", "block")
                                .style("font-size", "0.875rem")
                                .style("color", "#9ca3af")
                                .style("margin-bottom", "0.25rem")
                                .text("Username")
                            }),
                            html!("input" => web_sys::HtmlInputElement, {
                                .style("width", "100%")
                                .style("background", "#111827")
                                .style("border", "1px solid #374151")
                                .style("border-radius", "0.25rem")
                                .style("padding", "0.5rem 0.75rem")
                                .style("color", "white")
                                .style("box-sizing", "border-box")
                                .attr("type", "text")
                                .attr("placeholder", "demo")
                                .event(bind_input_value(username.clone()))
                            }),
                        ])
                    }),
                    html!("div", {
                        .style("margin-bottom", "1.5rem")
                        .children(&mut [
                            html!("label", {
                                .style("display", "block")
                                .style("font-size", "0.875rem")
                                .style("color", "#9ca3af")
                                .style("margin-bottom", "0.25rem")
                                .text("Password")
                            }),
                            html!("input" => web_sys::HtmlInputElement, {
                                .style("width", "100%")
                                .style("background", "#111827")
                                .style("border", "1px solid #374151")
                                .style("border-radius", "0.25rem")
                                .style("padding", "0.5rem 0.75rem")
                                .style("color", "white")
                                .style("box-sizing", "border-box")
                                .attr("type", "password")
                                .attr("placeholder", "demo")
                                .event(bind_input_value(password.clone()))
                            }),
                        ])
                    }),
                    html!("button", {
                        .style("width", "100%")
                        .style("background", "#2563eb")
                        .style("color", "white")
                        .style("padding", "0.625rem")
                        .style("border-radius", "0.25rem")
                        .style("border", "none")
                        .style("cursor", "pointer")
                        .style("font-size", "1rem")
                        .text("Sign in")
                        .event(clone!(state, username, password, error_msg => move |_: events::Click| {
                            let state = state.clone();
                            let username = username.get_cloned();
                            let password = password.get_cloned();
                            let error_msg = error_msg.clone();
                            spawn_local(async move {
                                let req = LoginRequest { username, password };
                                match post_json::<_, LoginResponse>(
                                    &format!("{API_BASE}/auth/login"),
                                    &req,
                                    None,
                                ).await {
                                    Ok(response) => {
                                        error_msg.set(None);
                                        state.auth.login(response);
                                    }
                                    Err(_) => {
                                        error_msg.set(Some("Invalid username or password".into()));
                                    }
                                }
                            });
                        }))
                    }),
                    html!("p", {
                        .style("margin-top", "1rem")
                        .style("font-size", "0.75rem")
                        .style("color", "#6b7280")
                        .style("text-align", "center")
                        .text("Default credentials: demo / demo")
                    }),
                ])
            }),
        ])
    })
}
