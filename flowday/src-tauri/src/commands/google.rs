use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use keyring::Entry;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// ── Constants ──

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_CALENDAR_BASE: &str = "https://www.googleapis.com/calendar/v3";
const CALENDAR_SCOPE: &str = "https://www.googleapis.com/auth/calendar";
const KEYRING_SERVICE: &str = "com.flowday.timer";
const REDIRECT_URI: &str = "http://localhost:19872/callback";

// ── Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenData {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: Option<String>,
    pub summary: String,
    pub start: EventDateTime,
    pub end: EventDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDateTime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAccount {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUrlResponse {
    pub url: String,
    pub state: String,
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    #[allow(dead_code)]
    token_type: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

#[derive(Deserialize)]
struct CalendarListResponse {
    items: Option<Vec<CalendarEvent>>,
}

// ── Managed state ──

pub struct GoogleAuthState {
    /// Maps account email -> stored PKCE code_verifier for pending auth flows
    pending_verifiers: Arc<Mutex<HashMap<String, String>>>,
    /// OAuth config (client_id, client_secret)
    config: Arc<Mutex<Option<OAuthConfig>>>,
    http: Client,
}

impl GoogleAuthState {
    pub fn new() -> Self {
        Self {
            pending_verifiers: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(None)),
            http: Client::new(),
        }
    }
}

// ── Keyring helpers ──

fn token_keyring_key(email: &str) -> String {
    format!("google-oauth-{}", email)
}

fn store_token(email: &str, token: &TokenData) -> Result<(), String> {
    let key = token_keyring_key(email);
    let entry = Entry::new(KEYRING_SERVICE, &key).map_err(|e| format!("Keyring error: {}", e))?;
    let json = serde_json::to_string(token).map_err(|e| format!("Serialize error: {}", e))?;
    entry
        .set_password(&json)
        .map_err(|e| format!("Keyring store error: {}", e))?;
    Ok(())
}

fn load_token(email: &str) -> Result<Option<TokenData>, String> {
    let key = token_keyring_key(email);
    let entry = Entry::new(KEYRING_SERVICE, &key).map_err(|e| format!("Keyring error: {}", e))?;
    match entry.get_password() {
        Ok(json) => {
            let token: TokenData =
                serde_json::from_str(&json).map_err(|e| format!("Deserialize error: {}", e))?;
            Ok(Some(token))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Keyring read error: {}", e)),
    }
}

fn delete_token(email: &str) -> Result<(), String> {
    let key = token_keyring_key(email);
    let entry = Entry::new(KEYRING_SERVICE, &key).map_err(|e| format!("Keyring error: {}", e))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Keyring delete error: {}", e)),
    }
}

/// List all stored account emails from the accounts index in keyring.
fn load_account_index() -> Result<Vec<String>, String> {
    let entry = Entry::new(KEYRING_SERVICE, "account-index")
        .map_err(|e| format!("Keyring error: {}", e))?;
    match entry.get_password() {
        Ok(json) => {
            let emails: Vec<String> =
                serde_json::from_str(&json).map_err(|e| format!("Deserialize error: {}", e))?;
            Ok(emails)
        }
        Err(keyring::Error::NoEntry) => Ok(vec![]),
        Err(e) => Err(format!("Keyring read error: {}", e)),
    }
}

fn save_account_index(emails: &[String]) -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE, "account-index")
        .map_err(|e| format!("Keyring error: {}", e))?;
    let json = serde_json::to_string(emails).map_err(|e| format!("Serialize error: {}", e))?;
    entry
        .set_password(&json)
        .map_err(|e| format!("Keyring store error: {}", e))?;
    Ok(())
}

fn add_to_account_index(email: &str) -> Result<(), String> {
    let mut emails = load_account_index()?;
    if !emails.iter().any(|e| e == email) {
        emails.push(email.to_string());
        save_account_index(&emails)?;
    }
    Ok(())
}

fn remove_from_account_index(email: &str) -> Result<(), String> {
    let mut emails = load_account_index()?;
    emails.retain(|e| e != email);
    save_account_index(&emails)?;
    Ok(())
}

// ── PKCE helpers ──

fn generate_pkce() -> (String, String) {
    let mut rng = rand::thread_rng();
    let mut verifier_bytes = [0u8; 32];
    rng.fill(&mut verifier_bytes);
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (code_verifier, code_challenge)
}

fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let mut state_bytes = [0u8; 16];
    rng.fill(&mut state_bytes);
    URL_SAFE_NO_PAD.encode(state_bytes)
}

// ── Token refresh ──

async fn refresh_access_token(
    http: &Client,
    config: &OAuthConfig,
    refresh_token: &str,
) -> Result<TokenData, String> {
    let params = [
        ("client_id", config.client_id.as_str()),
        ("client_secret", config.client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = http
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {}", body));
    }

    let token_resp: GoogleTokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Token refresh parse error: {}", e))?;

    let expires_at = Utc::now().timestamp() + token_resp.expires_in - 60; // 60s buffer

    Ok(TokenData {
        access_token: token_resp.access_token,
        refresh_token: token_resp
            .refresh_token
            .or_else(|| Some(refresh_token.to_string())),
        expires_at,
    })
}

/// Get a valid access token for the given email, refreshing if needed.
async fn get_valid_token(
    http: &Client,
    config: &OAuthConfig,
    email: &str,
) -> Result<String, String> {
    let token = load_token(email)?
        .ok_or_else(|| format!("No stored credentials for {}", email))?;

    if Utc::now().timestamp() < token.expires_at {
        return Ok(token.access_token);
    }

    // Token expired, try to refresh
    let refresh_token = token
        .refresh_token
        .as_deref()
        .ok_or("No refresh token available, re-authentication required")?;

    let new_token = refresh_access_token(http, config, refresh_token).await?;
    store_token(email, &new_token)?;
    Ok(new_token.access_token)
}

// ── Tauri commands ──

#[tauri::command]
pub async fn google_set_oauth_config(
    auth: State<'_, GoogleAuthState>,
    client_id: String,
    client_secret: String,
) -> Result<(), String> {
    let mut config = auth.config.lock().await;
    *config = Some(OAuthConfig {
        client_id,
        client_secret,
    });
    Ok(())
}

#[tauri::command]
pub async fn google_get_auth_url(
    auth: State<'_, GoogleAuthState>,
) -> Result<AuthUrlResponse, String> {
    let config = auth.config.lock().await;
    let config = config
        .as_ref()
        .ok_or("OAuth config not set. Call google_set_oauth_config first.")?;

    let (code_verifier, code_challenge) = generate_pkce();
    let state = generate_state();

    // Store verifier keyed by state so we can match it on callback
    auth.pending_verifiers
        .lock()
        .await
        .insert(state.clone(), code_verifier);

    let url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}&code_challenge={}&code_challenge_method=S256",
        GOOGLE_AUTH_URL,
        urlencoding(&config.client_id),
        urlencoding(REDIRECT_URI),
        urlencoding(CALENDAR_SCOPE),
        urlencoding(&state),
        urlencoding(&code_challenge),
    );

    Ok(AuthUrlResponse { url, state })
}

#[tauri::command]
pub async fn google_exchange_code(
    auth: State<'_, GoogleAuthState>,
    code: String,
    state: String,
) -> Result<GoogleAccount, String> {
    let config_guard = auth.config.lock().await;
    let config = config_guard
        .as_ref()
        .ok_or("OAuth config not set")?
        .clone();
    drop(config_guard);

    // Retrieve and consume the PKCE verifier
    let code_verifier = auth
        .pending_verifiers
        .lock()
        .await
        .remove(&state)
        .ok_or("Invalid or expired auth state")?;

    // Exchange code for tokens
    let params = [
        ("client_id", config.client_id.as_str()),
        ("client_secret", config.client_secret.as_str()),
        ("code", code.as_str()),
        ("code_verifier", code_verifier.as_str()),
        ("redirect_uri", REDIRECT_URI),
        ("grant_type", "authorization_code"),
    ];

    let resp = auth
        .http
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {}", body));
    }

    let token_resp: GoogleTokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Token parse error: {}", e))?;

    let expires_at = Utc::now().timestamp() + token_resp.expires_in - 60;

    // Fetch user info to get email
    let user_info: GoogleUserInfo = auth
        .http
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("User info parse error: {}", e))?;

    let token_data = TokenData {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        expires_at,
    };

    // Store token in keychain
    store_token(&user_info.email, &token_data)?;
    add_to_account_index(&user_info.email)?;

    Ok(GoogleAccount {
        email: user_info.email,
        name: user_info.name,
        picture: user_info.picture,
    })
}

#[tauri::command]
pub async fn google_list_accounts() -> Result<Vec<String>, String> {
    load_account_index()
}

#[tauri::command]
pub async fn google_remove_account(email: String) -> Result<(), String> {
    delete_token(&email)?;
    remove_from_account_index(&email)?;
    Ok(())
}

#[tauri::command]
pub async fn google_is_authenticated(email: String) -> Result<bool, String> {
    match load_token(&email)? {
        Some(token) => Ok(token.refresh_token.is_some()),
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn google_fetch_events(
    auth: State<'_, GoogleAuthState>,
    email: String,
    start_date: String,
    end_date: String,
) -> Result<Vec<CalendarEvent>, String> {
    let config_guard = auth.config.lock().await;
    let config = config_guard.as_ref().ok_or("OAuth config not set")?.clone();
    drop(config_guard);

    let access_token = get_valid_token(&auth.http, &config, &email).await?;

    // Parse dates and convert to RFC3339
    let time_min = format!("{}T00:00:00Z", start_date);
    let time_max = format!("{}T23:59:59Z", end_date);

    let url = format!(
        "{}/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime&maxResults=250",
        GOOGLE_CALENDAR_BASE,
        urlencoding(&time_min),
        urlencoding(&time_max),
    );

    let resp = auth
        .http
        .get(&url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Calendar API request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Calendar API error ({}): {}", status, body));
    }

    let list: CalendarListResponse = resp
        .json()
        .await
        .map_err(|e| format!("Calendar parse error: {}", e))?;

    Ok(list.items.unwrap_or_default())
}

#[tauri::command]
pub async fn google_create_event(
    auth: State<'_, GoogleAuthState>,
    email: String,
    title: String,
    start_time: String,
    end_time: String,
    description: Option<String>,
) -> Result<CalendarEvent, String> {
    let config_guard = auth.config.lock().await;
    let config = config_guard.as_ref().ok_or("OAuth config not set")?.clone();
    drop(config_guard);

    let access_token = get_valid_token(&auth.http, &config, &email).await?;

    // Validate that start_time and end_time parse as RFC3339
    DateTime::parse_from_rfc3339(&start_time)
        .map_err(|e| format!("Invalid start_time (need RFC3339): {}", e))?;
    DateTime::parse_from_rfc3339(&end_time)
        .map_err(|e| format!("Invalid end_time (need RFC3339): {}", e))?;

    let event = CalendarEvent {
        id: None,
        summary: title,
        start: EventDateTime {
            date_time: Some(start_time),
            date: None,
            time_zone: None,
        },
        end: EventDateTime {
            date_time: Some(end_time),
            date: None,
            time_zone: None,
        },
        description,
        status: None,
    };

    let url = format!("{}/calendars/primary/events", GOOGLE_CALENDAR_BASE);

    let resp = auth
        .http
        .post(&url)
        .bearer_auth(&access_token)
        .json(&event)
        .send()
        .await
        .map_err(|e| format!("Create event request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Create event error ({}): {}", status, body));
    }

    let created: CalendarEvent = resp
        .json()
        .await
        .map_err(|e| format!("Create event parse error: {}", e))?;

    Ok(created)
}

// ── URL encoding helper ──

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pkce() {
        let (verifier, challenge) = generate_pkce();
        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());
        assert_ne!(verifier, challenge);

        // Verify challenge is SHA256 of verifier
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(hasher.finalize());
        assert_eq!(challenge, expected);
    }

    #[test]
    fn test_generate_state() {
        let s1 = generate_state();
        let s2 = generate_state();
        assert!(!s1.is_empty());
        assert_ne!(s1, s2); // statistically should never collide
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello+world");
        assert_eq!(
            urlencoding("https://example.com"),
            "https%3A%2F%2Fexample.com"
        );
    }

    #[test]
    fn test_token_keyring_key() {
        assert_eq!(
            token_keyring_key("user@gmail.com"),
            "google-oauth-user@gmail.com"
        );
    }

    #[test]
    fn test_calendar_event_serialization() {
        let event = CalendarEvent {
            id: Some("abc123".into()),
            summary: "Meeting".into(),
            start: EventDateTime {
                date_time: Some("2026-04-06T10:00:00Z".into()),
                date: None,
                time_zone: None,
            },
            end: EventDateTime {
                date_time: Some("2026-04-06T11:00:00Z".into()),
                date: None,
                time_zone: None,
            },
            description: None,
            status: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"summary\":\"Meeting\""));
        // None fields should be absent due to skip_serializing_if
        assert!(!json.contains("\"description\""));
    }

    #[tokio::test]
    async fn test_google_auth_state_new() {
        let state = GoogleAuthState::new();
        let verifiers = state.pending_verifiers.lock().await;
        assert!(verifiers.is_empty());
        let config = state.config.lock().await;
        assert!(config.is_none());
    }
}
