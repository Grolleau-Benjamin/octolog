use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSpec {
    pub path: String,
    pub baud: Option<u32>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPortSpec {
    pub path: String,
    pub baud: u32,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortSpecParseError {
    EmptySpec,
    MissingPath,
    InvalidBaud { value: String },
}

impl fmt::Display for PortSpecParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySpec => write!(f, "empty port spec"),
            Self::MissingPath => write!(f, "missing port path"),
            Self::InvalidBaud { value } => write!(f, "invalid baudrate '{}'", value),
        }
    }
}

impl std::error::Error for PortSpecParseError {}

impl PortSpec {
    pub fn resolve(self, fallback_baud: u32) -> ResolvedPortSpec {
        ResolvedPortSpec {
            path: self.path,
            baud: self.baud.unwrap_or(fallback_baud),
            alias: self.alias,
        }
    }
}

impl FromStr for PortSpec {
    type Err = PortSpecParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.trim();
        if raw.is_empty() {
            return Err(PortSpecParseError::EmptySpec);
        }

        let mut parts = raw.splitn(3, ':');
        let path = parts.next().unwrap_or("").trim();
        if path.is_empty() {
            return Err(PortSpecParseError::MissingPath);
        }

        let p2 = parts.next().map(str::trim).filter(|v| !v.is_empty());
        let p3 = parts.next().map(str::trim).filter(|v| !v.is_empty());

        // Accepted forms:
        // - path
        // - path:baud
        // - path:alias
        // - path:baud:alias
        let (baud, alias) = match (p2, p3) {
            (None, None) => (None, None),
            (Some(x), None) => {
                if let Ok(b) = x.parse::<u32>() {
                    (Some(b), None)
                } else {
                    (None, Some(x.to_string()))
                }
            }
            (Some(x), Some(y)) => {
                let b = x
                    .parse::<u32>()
                    .map_err(|_| PortSpecParseError::InvalidBaud {
                        value: x.to_string(),
                    })?;
                (Some(b), Some(y.to_string()))
            }
            (None, Some(_)) => unreachable!(),
        };

        Ok(Self {
            path: path.to_string(),
            baud: baud,
            alias: alias,
        })
    }
}
