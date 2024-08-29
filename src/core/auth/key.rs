use rand::{thread_rng, Rng};

use crate::error::Result;
use crate::models::key::KeyToCreate;
use crate::{models, DbState};

pub fn create(db_state: &DbState, description: &Option<String>) -> Result<String> {
    let access_key = generate_key();
    let secret_key = generate_key();

    let _ = models::key::create(
        db_state,
        &KeyToCreate {
            access: access_key.clone(),
            secret: secret_key,
            description: description.clone(),
            expired_at: None,
        },
    );

    Ok(access_key)
}

pub fn read_all(db_state: &DbState) {
    // TODO: Read all tokens from database
}

pub fn expire(db_state: &DbState, id: u32) {
    // TODO: Update expired_at field in database
}

fn generate_key() -> String {
    const TOKEN_CHARS: &[u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*_-";

    let mut rng = thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..TOKEN_CHARS.len());
            TOKEN_CHARS[idx] as char
        })
        .collect()
}
