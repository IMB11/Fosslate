use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum EmailDeliveryClient {
    Resend(ResendClient),
    Static(StaticEmailDeliveryClient),
}

impl EmailDeliveryClient {
    pub fn resend(api_url: impl Into<String>) -> Self {
        Self::Resend(ResendClient::new(api_url))
    }

    pub fn static_success(message_id: impl Into<String>) -> Self {
        Self::Static(StaticEmailDeliveryClient::success(message_id))
    }

    pub fn static_failure(message: impl Into<String>) -> Self {
        Self::Static(StaticEmailDeliveryClient::failure(message))
    }

    pub async fn send_test_email(
        &self,
        request: ResendEmailRequest<'_>,
    ) -> Result<String, EmailDeliveryError> {
        match self {
            Self::Resend(client) => client.send_test_email(request).await,
            Self::Static(client) => client.send_test_email().await,
        }
    }

    pub async fn send_email(
        &self,
        request: ResendSendEmail<'_>,
    ) -> Result<String, EmailDeliveryError> {
        match self {
            Self::Resend(client) => client.send_email(request).await,
            Self::Static(client) => client.send_test_email().await,
        }
    }
}

#[derive(Clone)]
pub struct ResendClient {
    http: reqwest::Client,
    api_url: String,
}

impl ResendClient {
    fn new(api_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_url: api_url.into(),
        }
    }

    async fn send_test_email(
        &self,
        request: ResendEmailRequest<'_>,
    ) -> Result<String, EmailDeliveryError> {
        self.send_email(ResendSendEmail {
            api_key: request.api_key,
            from_name: request.from_name,
            from_email: request.from_email,
            recipient: request.test_recipient,
            subject: "Fosslate email delivery test",
            html: "<strong>Fosslate email delivery is configured.</strong>",
        })
        .await
    }

    async fn send_email(&self, request: ResendSendEmail<'_>) -> Result<String, EmailDeliveryError> {
        let response = self
            .http
            .post(&self.api_url)
            .bearer_auth(request.api_key)
            .json(&ResendSendEmailRequest {
                from: format!("{} <{}>", request.from_name, request.from_email),
                to: vec![request.recipient.to_owned()],
                subject: request.subject.to_owned(),
                html: request.html.to_owned(),
            })
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(EmailDeliveryError::Rejected(format!(
                "Resend returned HTTP {status}: {body}"
            )));
        }

        let body: ResendSendEmailResponse =
            serde_json::from_str(&body).map_err(EmailDeliveryError::InvalidResponse)?;

        body.id
            .filter(|id| !id.trim().is_empty())
            .ok_or(EmailDeliveryError::MissingMessageId)
    }
}

#[derive(Clone)]
pub struct StaticEmailDeliveryClient {
    result: StaticEmailDeliveryResult,
}

impl StaticEmailDeliveryClient {
    fn success(message_id: impl Into<String>) -> Self {
        Self {
            result: StaticEmailDeliveryResult::Success(message_id.into()),
        }
    }

    fn failure(message: impl Into<String>) -> Self {
        Self {
            result: StaticEmailDeliveryResult::Failure(message.into()),
        }
    }

    async fn send_test_email(&self) -> Result<String, EmailDeliveryError> {
        match &self.result {
            StaticEmailDeliveryResult::Success(message_id) => Ok(message_id.clone()),
            StaticEmailDeliveryResult::Failure(message) => {
                Err(EmailDeliveryError::Rejected(message.clone()))
            }
        }
    }
}

#[derive(Clone)]
enum StaticEmailDeliveryResult {
    Success(String),
    Failure(String),
}

pub struct ResendEmailRequest<'a> {
    pub api_key: &'a str,
    pub from_name: &'a str,
    pub from_email: &'a str,
    pub test_recipient: &'a str,
}

pub struct ResendSendEmail<'a> {
    pub api_key: &'a str,
    pub from_name: &'a str,
    pub from_email: &'a str,
    pub recipient: &'a str,
    pub subject: &'a str,
    pub html: &'a str,
}

#[derive(Debug, Serialize)]
struct ResendSendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

#[derive(Debug, Deserialize)]
struct ResendSendEmailResponse {
    id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailDeliveryError {
    #[error("email delivery request failed")]
    Request(#[from] reqwest::Error),
    #[error("email delivery provider rejected the request: {0}")]
    Rejected(String),
    #[error("email delivery provider returned invalid JSON")]
    InvalidResponse(serde_json::Error),
    #[error("email delivery provider did not return a message ID")]
    MissingMessageId,
}
