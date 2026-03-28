use std::rc::Rc;

use app_core::dto::LoginResponse;
use futures_signals::signal::Mutable;

#[derive(Clone)]
pub struct AppState {
    pub auth: Rc<AuthState>,
}

pub struct AuthState {
    pub token: Mutable<Option<String>>,
    pub user_id: Mutable<Option<String>>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            token: Mutable::new(None),
            user_id: Mutable::new(None),
        }
    }

    pub fn login(&self, response: LoginResponse) {
        self.token.set(Some(response.token));
        self.user_id.set(Some(response.user_id));
    }

    pub fn logout(&self) {
        self.token.set(None);
        self.user_id.set(None);
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            auth: Rc::new(AuthState::new()),
        }
    }
}
