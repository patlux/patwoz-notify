use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use web_push::{ContentEncoding, VapidSignatureBuilder, WebPushMessageBuilder, URL_SAFE_NO_PAD};
use web_push::{WebPushError, WebPushMessage};

use crate::{notification::Notification, subscribe_data::SubscribeData};

#[derive(Serialize, Deserialize)]
pub struct Subscription {
    pub id: i64,
    pub data: sqlx::types::Json<SubscribeData>,
}

impl Subscription {
    pub fn create_notification(
        self: &Self,
        private_key: &str,
        notification: &Notification,
    ) -> Result<WebPushMessage, WebPushError> {
        let subscription_info = web_push::SubscriptionInfo::new(
            &self.data.endpoint,
            &self.data.keys.p256dh,
            &self.data.keys.auth,
        );

        let sig_builder = VapidSignatureBuilder::from_base64(
            // &app_state.vapid_private_key,
            &private_key,
            URL_SAFE_NO_PAD,
            &subscription_info,
        )?
        .build()
        .expect("Failed to create signature.");

        let mut builder = WebPushMessageBuilder::new(&subscription_info);

        let body = serde_json::to_string(&notification).expect("Failed to serialize data.");
        let bb = body.into_bytes();

        builder.set_payload(ContentEncoding::Aes128Gcm, &bb);
        builder.set_vapid_signature(sig_builder);

        builder.build()
    }

    pub async fn find_by_subscribe_data(
        pool: &Pool<Sqlite>,
        subscribe_data: &SubscribeData,
    ) -> anyhow::Result<Self> {
        let result = sqlx::query_as!(
            Subscription,
            r#"
            SELECT id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscriptions
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
}
