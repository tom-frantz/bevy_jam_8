use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DeError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse toml: {0}")]
    FailedToParseToml(#[from] toml::de::Error),
}
