use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use web_push::{ContentEncoding, VapidSignatureBuilder, WebPushMessageBuilder, URL_SAFE_NO_PAD};
use web_push::{WebPushError, WebPushMessage};

use crate::{notification::Notification, subscribe_data::SubscribeData};

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: i64,
    pub data: sqlx::types::Json<SubscribeData>,
    pub device_id: String,
}

impl Subscription {
    pub fn create_notification(
        &self,
        private_key: &str,
        notification: &Notification,
    ) -> Result<WebPushMessage, WebPushError> {
        let subscription_info = web_push::SubscriptionInfo::new(
            &self.data.endpoint,
            &self.data.keys.p256dh,
            &self.data.keys.auth,
        );

        let sig_builder =
            VapidSignatureBuilder::from_base64(private_key, URL_SAFE_NO_PAD, &subscription_info)?
                .build()?;

        let mut builder = WebPushMessageBuilder::new(&subscription_info);

        let body = serde_json::to_string(&notification)?;
        let bb = body.into_bytes();

        builder.set_payload(ContentEncoding::Aes128Gcm, &bb);
        builder.set_vapid_signature(sig_builder);

        builder.build()
    }

    // pub async fn send_notification(
    //     &self,
    //     private_key: &str,
    //     notification: &Notification,
    // ) -> anyhow::Result<()> {
    //     notification.send(private_key, self).await
    // }

    pub async fn find_by_subscribe_data(
        pool: &Pool<Sqlite>,
        subscribe_data: &SubscribeData,
    ) -> anyhow::Result<Self> {
        let result = sqlx::query_as!(
            Subscription,
            r#"
            SELECT id, device_id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscription
            WHERE
                json_extract(data, '$.keys.auth') == $1 
                AND
                json_extract(data, '$.keys.p256dh') == $2
                AND
                json_extract(data, '$.endpoint') == $3
            LIMIT 1
        "#,
            subscribe_data.keys.auth,
            subscribe_data.keys.p256dh,
            subscribe_data.endpoint,
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn new_with_device_id(
        pool: &Pool<Sqlite>,
        device_id: String,
    ) -> Result<Self, anyhow::Error> {
        let subscription = sqlx::query_as!(
            Subscription,
            r#"SELECT id, device_id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscription WHERE device_id = $1"#,
            device_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(subscription)
    }
}
