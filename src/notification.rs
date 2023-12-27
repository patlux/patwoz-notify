use serde::Serialize;
use web_push::WebPushClient;
use web_push::{HyperWebPushClient, WebPushError, WebPushMessage};

use crate::subscription::Subscription;

#[derive(Serialize, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
}

impl Notification {
    pub fn build(
        self: &Self,
        private_key: &str,
        subscription: &Subscription,
    ) -> Result<WebPushMessage, WebPushError> {
        subscription.create_notification(private_key, self)
    }

    pub async fn send(
        self: &Self,
        private_key: &str,
        subscription: &Subscription,
    ) -> anyhow::Result<()> {
        HyperWebPushClient::new()
            .send(
                self.build(private_key, subscription)
                    .expect("Failed to create notification."),
            )
            .await
            .expect("Failed to send notification.");

        Ok(())
    }
}