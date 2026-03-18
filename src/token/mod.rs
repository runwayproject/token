#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub version: u32,
    pub rid: String,
    pub relay: String,
    pub keyserver: Option<(String, String)>,
    pub keypackage: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidScheme,
    InvalidVersion,
    MissingRidRelay,
    InvalidKeyserverFormat,
    UnknownSegment(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidScheme => write!(f, "invalid scheme"),
            ParseError::InvalidVersion => write!(f, "invalid version"),
            ParseError::MissingRidRelay => write!(f, "missing rid@relay segment"),
            ParseError::InvalidKeyserverFormat => {
                write!(f, "invalid keyserver format (expected handle@host)")
            }
            ParseError::UnknownSegment(s) => write!(f, "unknown segment: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<&str> for ParseError {
    fn from(s: &str) -> Self {
        ParseError::UnknownSegment(s.to_string())
    }
}

pub fn parse_token(s: &str) -> Result<Token, ParseError> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() < 3 {
        return Err(ParseError::MissingRidRelay);
    }

    if parts[0] != "runway" {
        return Err(ParseError::InvalidScheme);
    }

    let ver_part = parts[1];
    if !ver_part.starts_with('v') {
        return Err(ParseError::InvalidVersion);
    }

    let version: u32 = ver_part[1..]
        .parse()
        .map_err(|_| ParseError::InvalidVersion)?;

    let rid_relay = parts[2];
    let at = rid_relay.rfind('@').ok_or(ParseError::MissingRidRelay)?;
    let rid = &rid_relay[..at];
    let relay = &rid_relay[at + 1..];

    let mut keyserver: Option<(String, String)> = None;
    let mut keypackage: Option<String> = None;

    for seg in parts.iter().skip(3) {
        if seg.is_empty() {
            continue;
        }
        if seg.contains('@') {
            let at = seg.rfind('@').ok_or(ParseError::InvalidKeyserverFormat)?;
            let handle = seg[..at].to_string();
            let host = seg[at + 1..].to_string();
            keyserver = Some((handle, host));
            continue;
        }
        if keypackage.is_none() {
            keypackage = Some(seg.to_string());
            continue;
        }
        return Err(ParseError::UnknownSegment(seg.to_string()));
    }

    Ok(Token {
        version,
        rid: rid.to_string(),
        relay: relay.to_string(),
        keyserver,
        keypackage,
    })
}
