use std::sync::Arc;
use crate::config::Config;
use crate::common::{AppResult, AppError};
use lettre::{
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

#[derive(Clone, Debug)]
pub struct Mailer {
    transport: Option<Arc<SmtpTransport>>,
    from_address: Option<String>,
}

impl Mailer {
    pub fn new(config: &Config) -> Self {
        if let Some(host) = &config.smtp_host {
            let port = config.smtp_port.unwrap_or(587);
            let user = config.smtp_user.clone().unwrap_or_default();
            let password = config.smtp_password.clone().unwrap_or_default();
            let from_address = config.smtp_from.clone();

            let creds = Credentials::new(user, password);
            let transport = SmtpTransport::relay(host)
                .expect("valid SMTP host")
                .port(port)
                .credentials(creds)
                .build();

            Self {
                transport: Some(Arc::new(transport)),
                from_address,
            }
        } else {
            Self {
                transport: None,
                from_address: None,
            }
        }
    }

    pub async fn send_otp(&self, email: &str, purpose: &str, otp: &str) {
        tracing::info!(%email, %purpose, %otp, "generated otp");
        let subject = format!("Your OTP for {}", purpose);
        let body = format!("Your OTP is: {}\nThis OTP expires shortly.", otp);
        if let Err(e) = self.send_email(email, &subject, &body).await {
            tracing::error!(error = %e, "failed_to_send_otp_email");
        }
    }

    pub async fn send_email(&self, recipient: &str, subject: &str, body: &str) -> AppResult<()> {
        if let Some(transport) = &self.transport {
            let from = self.from_address.as_deref().unwrap_or("noreply@forkfit.com");
            
            let email_msg = Message::builder()
                .from(from.parse().map_err(|e| AppError::BadRequest(format!("invalid from email: {e}"))).unwrap())
                .to(recipient.parse().map_err(|e| AppError::BadRequest(format!("invalid recipient email: {e}"))).unwrap())
                .subject(subject)
                .body(body.to_string())
                .map_err(|e| AppError::BadRequest(format!("failed to construct email message: {e}")))?;

            let transport_clone = transport.clone();
            tokio::task::spawn_blocking(move || {
                transport_clone.send(&email_msg)
            })
            .await
            .map_err(|e| AppError::Internal(format!("blocking task join failed: {e}")))?
            .map_err(|e| AppError::Internal(format!("failed to deliver email via SMTP: {e}")))?;

            tracing::info!(%recipient, %subject, "email_delivered_via_smtp");
        } else {
            tracing::info!(%recipient, %subject, %body, "email_console_log_fallback");
        }
        Ok(())
    }
}

impl Default for Mailer {
    fn default() -> Self {
        Self {
            transport: None,
            from_address: None,
        }
    }
}
