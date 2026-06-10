#[derive(Clone, Debug)]
pub struct Mailer;

impl Mailer {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_otp(&self, email: &str, purpose: &str, otp: &str) {
        tracing::info!(%email, %purpose, %otp, "generated otp");
    }
}

impl Default for Mailer {
    fn default() -> Self {
        Self::new()
    }
}
