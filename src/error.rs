use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub enum QRErrors {
    QrSerachError, // just not found qr in the dir
    QrDetectError, // failed to detect qr on image
    QrAlreadyFound,
}

impl fmt::Display for QRErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QRErrors::QrSerachError => write!(f, "failed to find qr"),
            QRErrors::QrDetectError => write!(f, "failed to detect qr"),
            QRErrors::QrAlreadyFound => write!(f, "qr already found, not need to search it"),
        }
    }
}

impl std::error::Error for QRErrors {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
