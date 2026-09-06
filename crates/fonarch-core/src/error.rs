use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("livetype cache not found")]
    NotFound,
    #[error("entitlements catalog is not a typekitSyncState document: {}", .0.display())]
    BadCatalog(PathBuf),
    #[error("could not find the Desktop folder")]
    NoDesktop,
    #[error("failed to parse entitlements.xml: {0}")]
    Xml(String),
    #[error("failed to zip archive: {0}")]
    Zip(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
