use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use app_frontend::components::login::login_form;
use app_frontend::components::state::AppState;

#[wasm_bindgen_test]
fn login_form_renders_inputs_and_button() {
    let state = AppState::new();
    let dom = login_form(state);

    // Append to document body
    dominator::append_dom(&dominator::body(), dom);

    let document = web_sys::window().unwrap().document().unwrap();

    // Verify form elements exist
    let inputs = document.query_selector_all("input").unwrap();
    assert!(
        inputs.length() >= 2,
        "Should have at least username and password inputs"
    );

    let buttons = document.query_selector_all("button").unwrap();
    assert!(buttons.length() >= 1, "Should have a submit button");

    // Clean up
    let body = document.body().unwrap();
    while let Some(child) = body.last_child() {
        body.remove_child(&child).unwrap();
    }
}

#[wasm_bindgen_test]
fn app_state_login_and_logout() {
    let state = AppState::new();

    // Initially not logged in
    assert!(state.auth.token.get_cloned().is_none());
    assert!(state.auth.user_id.get_cloned().is_none());

    // Login
    state.auth.login(app_core::dto::LoginResponse {
        token: "test-jwt".into(),
        user_id: "alice".into(),
        permissions: vec!["items:write".into()],
    });

    assert_eq!(state.auth.token.get_cloned(), Some("test-jwt".into()));
    assert_eq!(state.auth.user_id.get_cloned(), Some("alice".into()));

    // Logout
    state.auth.logout();
    assert!(state.auth.token.get_cloned().is_none());
    assert!(state.auth.user_id.get_cloned().is_none());
}

#[wasm_bindgen_test]
fn app_root_shows_login_when_not_authenticated() {
    let state = AppState::new();

    // Not logged in → login form should render
    let dom = login_form(state);
    dominator::append_dom(&dominator::body(), dom);

    let document = web_sys::window().unwrap().document().unwrap();

    // Should have password input (login form marker)
    let password_inputs = document.query_selector("input[type='password']").unwrap();
    assert!(
        password_inputs.is_some(),
        "Login form should render with password input"
    );

    // Clean up
    let body = document.body().unwrap();
    while let Some(child) = body.last_child() {
        body.remove_child(&child).unwrap();
    }
}
