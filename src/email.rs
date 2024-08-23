const MAILGUN_API_URL: &str = "https://api.mailgun.net/v3/email.b01lers.com";

pub struct EmailClient {
    mailgun_token: String,
}

impl EmailClient {
    pub fn new(mailgun_token: String) -> Self {
        EmailClient {
            mailgun_token,
        }
    }

    pub async fn send_email(&self, dest_addr: &str, title: &str, body: &str) -> anyhow::Result<()> {
        let client = reqwest::Client::new();

        let result = client.post(format!("{MAILGUN_API_URL}/messages"))
            .basic_auth("api", Some(&self.mailgun_token))
            .form(&[
                ("from", "Purdue Capture The Flag Team <b01lers@b01lers.com>"),
                ("to", dest_addr),
                ("subject", title),
                ("html", body),
            ])
            .send()
            .await?;

        if !result.status().is_success() {
            Err(anyhow::anyhow!("Failed to send verification email: {result:?}"))
        } else {
            Ok(())
        }
    }
}