use quick_xml::events::attributes::AttrError;

#[derive(Debug)]
pub enum Error {
    Input,
    InvalidFormat,
    InvalidData,
}

#[derive(Debug)]
pub enum InternalError {
    Io(std::io::Error),
    Xml(String),
    InvalidTrackPoint(String),
}

impl From<std::io::Error> for InternalError {
    fn from(value: std::io::Error) -> Self {
        InternalError::Io(value)
    }
}

impl From<quick_xml::Error> for InternalError {
    fn from(value: quick_xml::Error) -> Self {
        InternalError::Xml(value.to_string())
    }
}

impl From<AttrError> for InternalError {
    fn from(e: AttrError) -> Self {
        InternalError::Xml(e.to_string())
    }
}

impl From<InternalError> for Error {
    fn from(e: InternalError) -> Self {
        match e {
            InternalError::Io(_) => Error::Input,
            InternalError::Xml(_) => Error::InvalidFormat,
            InternalError::InvalidTrackPoint(_) => Error::InvalidData,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Input => write!(f, "invalid input"),
            Error::InvalidFormat => write!(f, "invalid GPX format"),
            Error::InvalidData => write!(f, "invalid GPX data"),
        }
    }
}

impl std::error::Error for Error {}
