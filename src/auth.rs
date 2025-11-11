use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Session duration: 10 minutes
const SESSION_DURATION: Duration = Duration::from_secs(10 * 60);

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub username: String,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
}

impl Session {
    pub fn new(username: String) -> Self {
        let now = SystemTime::now();
        Self {
            user_id: Uuid::new_v4().to_string(),
            username,
            created_at: now,
            expires_at: now + SESSION_DURATION,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// In-memory session store
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(&self, username: String) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = Session::new(username);
        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);
        session_id
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            if session.is_expired() {
                sessions.remove(session_id);
                None
            } else {
                Some(session.clone())
            }
        } else {
            None
        }
    }

    pub fn delete_session(&self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);
    }

    pub fn cleanup_expired(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|_, session| !session.is_expired());
    }
}

/// Global session store
static SESSION_STORE: OnceLock<SessionStore> = OnceLock::new();

pub fn get_session_store() -> &'static SessionStore {
    SESSION_STORE.get_or_init(|| SessionStore::new())
}

/// Extract session ID from cookie header
fn extract_session_id(request: &Request) -> Option<String> {
    request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "session_id" {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
        })
}

/// Auth middleware - protects routes
pub async fn auth_middleware(request: Request, next: Next) -> Response {
    // Extract session ID from cookie
    let session_id = extract_session_id(&request);

    // Check if session is valid
    if let Some(sid) = session_id {
        let store = get_session_store();
        if store.get_session(&sid).is_some() {
            // Valid session, continue
            return next.run(request).await;
        }
    }

    // No valid session, redirect to login
    // Check if this is an HTMX request
    let is_htmx = request
        .headers()
        .get("hx-request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false);

    if is_htmx {
        // For HTMX requests, use HX-Redirect header
        axum::response::Response::builder()
            .status(200)
            .header("HX-Redirect", "/login")
            .body(axum::body::Body::empty())
            .unwrap()
    } else {
        // For regular requests, use standard redirect
        Redirect::to("/login").into_response()
    }
}

/// Public routes middleware - redirect to / if already logged in
pub async fn public_only_middleware(request: Request, next: Next) -> Response {
    let session_id = extract_session_id(&request);

    if let Some(sid) = session_id {
        let store = get_session_store();
        if store.get_session(&sid).is_some() {
            // Already logged in, redirect to home
            return Redirect::to("/").into_response();
        }
    }

    // Not logged in, continue
    next.run(request).await
}
