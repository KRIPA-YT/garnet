use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use rand::{RngExt as _, rng};
use sha2::Digest;

pub(crate) struct TokenPair {
    pub access: Token<Limited>,
    pub refresh: Token<Limited>,
}

pub(crate) trait Expiry: Send + Sync {}

pub(crate) type Limited = DateTime<Utc>;
pub(crate) struct Unlimited;

impl Expiry for Limited {}
impl Expiry for Unlimited {}

#[derive(sqlx::FromRow)]
pub(crate) struct Token<E: Expiry> {
    pub token: String,
    pub expiry: E,
}

impl Token<Unlimited> {
    pub(crate) fn random() -> Self {
        let mut rng = rng();
        let token = (0..32)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                #[allow(clippy::indexing_slicing)]
                char::from(CHARSET[idx])
            })
            .collect();
        Self {
            token,
            expiry: Unlimited,
        }
    }

    pub(crate) fn from_base64(base64: &str) -> Option<Self> {
        STANDARD
            .decode(base64)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .map(|token| Self {
                token,
                expiry: Unlimited,
            })
    }

    pub(crate) fn limited(self, expiry: Limited) -> Token<Limited> {
        Token {
            token: self.token,
            expiry,
        }
    }
}

impl<E: Expiry> Token<E> {
    pub(crate) fn base64_encode(&self) -> String {
        STANDARD.encode(&self.token)
    }

    pub(crate) fn hash(&self) -> Vec<u8> {
        sha2::Sha256::digest(self.token.as_bytes()).to_vec()
    }
}

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~";
