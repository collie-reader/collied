#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid value `{0}`")]
    InvalidValue(String),

    #[error("invalid key `{0}` for `{1}`")]
    InvalidEnumKey(String, String),

    #[error("invalid feed link `{0}`")]
    InvalidFeedLink(String),

    #[error("forbidden")]
    Forbidden,

    #[error("failed to parse syndication feed")]
    SyndicationParsingFailure,

    #[error("failed to fetch feed: {0}")]
    FetchFeedFailure(String),

    #[error("failed to fetch feed items: {0}")]
    FetchFeedItemsFailure(String),

    #[error("empty string")]
    EmptyString,

    #[error("unknown")]
    Unknown,

    #[error(transparent)]
    CollieCore {
        #[from]
        source: collie::error::Error,
    },

    #[error(transparent)]
    CollieAuth {
        #[from]
        source: collie::auth::error::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
