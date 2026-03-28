use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response};

pub const API_BASE: &str = "/api";

pub async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, JsValue> {
    let window = web_sys::window().unwrap_throw();
    let resp: Response = JsFuture::from(window.fetch_with_str(url)).await?.into();
    let json = JsFuture::from(resp.json()?).await?;
    Ok(serde_wasm_bindgen::from_value(json)?)
}

pub async fn post_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
    url: &str,
    body: &T,
    token: Option<&str>,
) -> Result<R, JsValue> {
    let window = web_sys::window().unwrap_throw();
    let opts = RequestInit::new();
    opts.set_method("POST");

    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    if let Some(token) = token {
        headers.set("Authorization", &format!("Bearer {token}"))?;
    }
    opts.set_headers(&headers);

    let body_str = serde_json::to_string(body).map_err(|e| JsValue::from_str(&e.to_string()))?;
    opts.set_body(&JsValue::from_str(&body_str));

    let request = Request::new_with_str_and_init(url, &opts)?;
    let resp: Response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .into();

    if !resp.ok() {
        return Err(JsValue::from_str(&format!("HTTP {}", resp.status())));
    }

    let json = JsFuture::from(resp.json()?).await?;
    Ok(serde_wasm_bindgen::from_value(json)?)
}

pub async fn delete_json(url: &str, token: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap_throw();
    let opts = RequestInit::new();
    opts.set_method("DELETE");

    let headers = Headers::new()?;
    headers.set("Authorization", &format!("Bearer {token}"))?;
    opts.set_headers(&headers);

    let request = Request::new_with_str_and_init(url, &opts)?;
    let resp: Response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .into();

    if !resp.ok() {
        return Err(JsValue::from_str(&format!("HTTP {}", resp.status())));
    }

    Ok(())
}

pub fn bind_input_value(
    mutable: futures_signals::signal::Mutable<String>,
) -> impl FnMut(dominator::events::Input) {
    move |e: dominator::events::Input| {
        if let Some(target) = e.target() {
            if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                mutable.set(input.value());
            }
        }
    }
}
