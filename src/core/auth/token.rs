use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    models, DbState,
};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub iat: i64,
    pub exp: i64,
}

pub fn verify_token(db_state: &DbState, token: &str) -> Result<bool> {
    let validation = Validation::default();
    let secret_key = models::key::find_secret_by_access(db_state, token)?;
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_bytes()),
        &validation,
    ) {
        Ok(token) => {
            if token.claims.exp > Utc::now().timestamp() {
                Ok(true)
            } else {
                Err(Error::Unauthorized)
            }
        }
        Err(_) => Err(Error::Unauthorized),
    }
}

pub fn create_token(db_state: &DbState, access_key: &str) -> Result<String> {
    let secret_key = models::key::find_secret_by_access(db_state, access_key)?;

    let now = Utc::now().timestamp();
    let claims = Claims {
        iat: now,
        exp: now + 3600,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )?;

    Ok(token)
}
