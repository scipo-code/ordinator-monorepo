use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LocalAuthPayload
{
    pub client_id: String,
    pub client_secret: String,
}
