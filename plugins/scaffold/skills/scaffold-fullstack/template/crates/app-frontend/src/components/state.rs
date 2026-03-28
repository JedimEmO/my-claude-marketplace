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

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::dto::LoginResponse;

    #[test]
    fn login_sets_token_and_user() {
        let state = AppState::new();
        assert!(state.auth.token.get_cloned().is_none());

        state.auth.login(LoginResponse {
            token: "jwt-123".into(),
            user_id: "alice".into(),
            permissions: vec!["items:write".into()],
        });

        assert_eq!(state.auth.token.get_cloned().unwrap(), "jwt-123");
        assert_eq!(state.auth.user_id.get_cloned().unwrap(), "alice");
    }

    #[test]
    fn logout_clears_state() {
        let state = AppState::new();
        state.auth.login(LoginResponse {
            token: "jwt-123".into(),
            user_id: "alice".into(),
            permissions: vec![],
        });

        state.auth.logout();

        assert!(state.auth.token.get_cloned().is_none());
        assert!(state.auth.user_id.get_cloned().is_none());
    }
}
