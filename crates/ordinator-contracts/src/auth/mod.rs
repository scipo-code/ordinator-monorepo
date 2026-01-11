use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::AssetNames;

#[derive(Serialize, TS, Deserialize)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct LoginResponse
{
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub role: String,
    pub assets: Vec<AssetNames>,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct AuthConfig
{
    pub provider: String,
}

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct LocalAuthPayload
{
    pub client_id: String,
    pub client_secret: String,
}
