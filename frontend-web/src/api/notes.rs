use crate::models::{CreateNoteRequest, Note};

use super::client::{ClientError, api_base, authed_client};

#[derive(Debug)]
pub enum NotesApiError {
    Client(ClientError),
    Request(reqwest::Error),
}

impl std::fmt::Display for NotesApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Client(error) => write!(f, "{error}"),
            Self::Request(error) => write!(f, "request failed: {error}"),
        }
    }
}

impl std::error::Error for NotesApiError {}

pub async fn fetch_timeline(
    token: &str,
    endpoint: &str,
    since_id: Option<&str>,
) -> Result<Vec<Note>, NotesApiError> {
    let client = authed_client(token).map_err(NotesApiError::Client)?;
    let mut request = client.get(format!(
        "{}/{}",
        api_base(),
        endpoint.trim_start_matches('/')
    ));
    if let Some(since_id) = since_id {
        request = request.query(&[("since_id", since_id)]);
    }
    request
        .send()
        .await
        .map_err(NotesApiError::Request)?
        .error_for_status()
        .map_err(NotesApiError::Request)?
        .json::<Vec<Note>>()
        .await
        .map_err(NotesApiError::Request)
}

pub async fn create_note(token: &str, body: &CreateNoteRequest) -> Result<Note, NotesApiError> {
    authed_client(token)
        .map_err(NotesApiError::Client)?
        .post(format!("{}/v1/notes", api_base()))
        .json(body)
        .send()
        .await
        .map_err(NotesApiError::Request)?
        .error_for_status()
        .map_err(NotesApiError::Request)?
        .json::<Note>()
        .await
        .map_err(NotesApiError::Request)
}
