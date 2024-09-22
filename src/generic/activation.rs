use crate::Error;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use std::env::var;

pub fn send_activation_email(email_address: &str, activation_string: &str) -> crate::Result<()> {
    // Send the E-mail, if this fails, the member creation will be rolled back by the database
    let email_dev_mode: bool = var("EMAIL_DEV_MODE")
        .unwrap_or("false".to_owned())
        .parse()
        .map_err(|_| Error::var_error("Development mode not properly set as true or false"))?;
    let email_from = var("EMAIL_FROM")?;
    let email_subject = var("EMAIL_REGISTRATION_SUBJECT")?;
    let email_body = var("EMAIL_REGISTRATION_BODY")?.replace("{}", &activation_string);
    let email_smtp_user = var("EMAIL_SMTP_USER")?;
    let email_smtp_password = var("EMAIL_SMTP_PASSWORD")?;
    let email_smtp_relay = var("EMAIL_SMTP_RELAY")?;
    let email_smtp_port = var("EMAIL_SMTP_PORT")
        .unwrap_or("587".to_owned())
        .parse()
        .map_err(|_| Error::var_error("Could not concert EMAIL_SMTP_PORT to port number"))?;
    let email = Message::builder()
        .from(email_from.parse()?)
        .to(email_address.parse()?)
        .subject(email_subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(email_body)?;

    let mut builder = SmtpTransport::relay(&email_smtp_relay)?.port(email_smtp_port);
    if !email_dev_mode {
        let smtp_relay_credentials = lettre::transport::smtp::authentication::Credentials::new(
            email_smtp_user,
            email_smtp_password,
        );
        builder = builder.credentials(smtp_relay_credentials)
    } else {
        builder = builder.tls(Tls::None)
    }
    let relay = builder.build();
    relay.send(&email)?;
    Ok(())
}
