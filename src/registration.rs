// SPDX-FileCopyrightText: 2026 roolrz
// SPDX-License-Identifier: Apache-2.0

//! Shared registration validation, including the complete encoded frame.

use crate::{Source, Stream};

pub const MAX_BYTES: usize = 512;
pub const MAX_COMPONENT_BYTES: usize = 255;
const MAGIC: &[u8; 4] = b"LOG2";
const RAW_MAGIC: &[u8; 4] = b"LOGS";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidComponent,
    TooLarge,
    InvalidEncoding,
}

fn validate(source: &Source) -> Result<(), Error> {
    if source.component.is_empty()
        || source.component.len() > MAX_COMPONENT_BYTES
        || source.component.chars().any(char::is_control)
    {
        return Err(Error::InvalidComponent);
    }
    Ok(())
}

/// Validate before serialization and include the magic and JSON escaping in
/// the wire limit. No capability is moved for a locally rejected source.
pub fn encode(source: &Source) -> Result<Vec<u8>, Error> {
    validate(source)?;
    let mut bytes = MAGIC.to_vec();
    bytes.extend(serde_json::to_vec(source).map_err(|_| Error::InvalidEncoding)?);
    if bytes.len() > MAX_BYTES {
        return Err(Error::TooLarge);
    }
    Ok(bytes)
}

/// Decode both structured registration and the raw-console registration form.
/// Callers must separately validate the transferred capability count and rights.
pub fn decode(bytes: &[u8]) -> Result<Source, Error> {
    if bytes.len() > MAX_BYTES {
        return Err(Error::TooLarge);
    }
    let source = if let Some(body) = bytes.strip_prefix(MAGIC) {
        serde_json::from_slice(body).map_err(|_| Error::InvalidEncoding)?
    } else if let Some(body) = bytes.strip_prefix(RAW_MAGIC) {
        Source {
            component: std::str::from_utf8(body)
                .map_err(|_| Error::InvalidEncoding)?
                .into(),
            generation: 0,
            stream: Stream::Log,
        }
    } else {
        return Err(Error::InvalidEncoding);
    };
    validate(&source)?;
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(component: impl Into<String>) -> Source {
        Source {
            component: component.into(),
            generation: u64::MAX,
            stream: Stream::Stderr,
        }
    }

    #[test]
    fn client_and_server_reject_all_control_characters() {
        for character in (0..=0x9f)
            .filter_map(char::from_u32)
            .filter(|c| c.is_control())
        {
            let source = source(format!("service{character}"));
            assert_eq!(encode(&source), Err(Error::InvalidComponent));
            let encoded = serde_json::to_vec(&source);
            assert!(encoded.is_ok());
            if let Ok(body) = encoded {
                let mut frame = MAGIC.to_vec();
                frame.extend(body);
                assert!(matches!(decode(&frame), Err(Error::InvalidComponent)));
            }
        }
    }

    #[test]
    fn component_limit_counts_utf8_bytes() {
        for name in ["".to_string(), "a".repeat(256), "é".repeat(128)] {
            assert_eq!(encode(&source(name)), Err(Error::InvalidComponent));
        }
        for name in ["a".repeat(255), format!("{}a", "é".repeat(127))] {
            let expected = source(name);
            let encoded = encode(&expected);
            assert!(encoded.is_ok());
            if let Ok(bytes) = encoded {
                let decoded = decode(&bytes);
                assert!(decoded.is_ok());
                if let Ok(actual) = decoded {
                    assert_eq!(actual.component, expected.component);
                    assert_eq!(actual.generation, expected.generation);
                    assert_eq!(actual.stream, expected.stream);
                }
            }
        }
    }

    #[test]
    fn exact_wire_limit_includes_json_escaping_and_prefix() {
        let overhead = MAGIC.len()
            + serde_json::to_vec(&source(""))
                .map(|bytes| bytes.len())
                .unwrap_or_default();
        assert!(overhead > MAGIC.len());
        let payload = MAX_BYTES - overhead;
        let name = format!("{}{}", "\\".repeat(payload / 2), "a".repeat(payload % 2));
        assert!(name.len() < MAX_COMPONENT_BYTES);
        let encoded = encode(&source(&name));
        assert!(encoded.is_ok());
        if let Ok(mut bytes) = encoded {
            assert_eq!(bytes.len(), MAX_BYTES);
            assert!(decode(&bytes).is_ok());
            bytes.push(b' ');
            assert!(matches!(decode(&bytes), Err(Error::TooLarge)));
        }
        assert_eq!(encode(&source(name + "a")), Err(Error::TooLarge));
        assert_eq!(encode(&source("\"".repeat(255))), Err(Error::TooLarge));
    }

    #[test]
    fn raw_registration_obeys_the_same_character_contract() {
        let valid = decode(b"LOGS/boot/init");
        assert!(valid.is_ok());
        if let Ok(source) = valid {
            assert_eq!(source.component, "/boot/init");
            assert_eq!(source.generation, 0);
            assert_eq!(source.stream, Stream::Log);
        }
        for bytes in [b"LOGS".as_slice(), b"LOGSbad\tname", b"LOGSbad\x7fname"] {
            assert!(matches!(decode(bytes), Err(Error::InvalidComponent)));
        }
        for bytes in [b"LOGS\xff".as_slice(), b"LOG2{}", b"LOG", b"NONE/name"] {
            assert!(matches!(decode(bytes), Err(Error::InvalidEncoding)));
        }
    }
}
