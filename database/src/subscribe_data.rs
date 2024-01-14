use serde::{Deserialize, Serialize};
use sqlx::{prelude::Type, Decode, Encode, Sqlite};

#[derive(Clone, Serialize, Deserialize)]
pub struct Keys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubscribeData {
    #[serde(rename = "expirationTime")]
    pub expiration_time: Option<i64>,
    pub endpoint: String,
    pub keys: Keys,
}

impl Encode<'_, Sqlite> for SubscribeData {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let value = serde_json::to_string(self).unwrap_or_default();
        <String as Encode<Sqlite>>::encode(value, buf)
    }
}

impl Decode<'_, Sqlite> for SubscribeData {
    fn decode(
        value: <Sqlite as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as Decode<Sqlite>>::decode(value)?;
        let value: SubscribeData = serde_json::from_str(value)?;
        Ok(value)
    }
}

impl Type<Sqlite> for SubscribeData {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}
