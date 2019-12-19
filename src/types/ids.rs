use super::{time::Seconds, types::Bytes};
use rand::RngCore;
use serde::{de, ser, Serializer};
use std::fmt;

/// Unique identifier of an operation within a trace
#[derive(Debug, PartialEq, Clone)]
pub enum SegmentId {
    #[doc(hidden)]
    New([u8; 8]),
    #[doc(hidden)]
    Rendered(String),
}

impl SegmentId {
    /// Generate a new random segment ID
    pub fn new() -> Self {
        let mut buf = [0; 8];
        rand::thread_rng().fill_bytes(&mut buf);
        SegmentId::New(buf)
    }
}

impl fmt::Display for SegmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentId::New(bytes) => write!(f, "{:x}", Bytes(bytes)),
            SegmentId::Rendered(value) => write!(f, "{}", value),
        }
    }
}

impl Default for SegmentId {
    fn default() -> Self {
        SegmentId::new()
    }
}

struct SegmentIdVisitor;

impl<'de> de::Visitor<'de> for SegmentIdVisitor {
    type Value = SegmentId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string value")
    }

    fn visit_str<E>(self, value: &str) -> Result<SegmentId, E>
    where
        E: de::Error,
    {
        Ok(SegmentId::Rendered(value.into()))
    }
}

impl ser::Serialize for SegmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> de::Deserialize<'de> for SegmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(SegmentIdVisitor)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TraceId {
    #[doc(hidden)]
    New(u64, [u8; 12]),
    #[doc(hidden)]
    Rendered(String),
}

impl TraceId {
    /// Generate a new random trace ID
    pub fn new() -> Self {
        let mut buf = [0; 12];
        rand::thread_rng().fill_bytes(&mut buf);
        TraceId::New(Seconds::now().trunc(), buf)
    }
}

impl Default for TraceId {
    fn default() -> Self {
        TraceId::new()
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceId::New(seconds, bytes) => write!(f, "1-{:08x}-{:x}", seconds, Bytes(bytes)),
            TraceId::Rendered(value) => write!(f, "{}", value),
        }
    }
}

struct TraceIdVisitor;

impl<'de> de::Visitor<'de> for TraceIdVisitor {
    type Value = TraceId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string value")
    }
    fn visit_str<E>(self, value: &str) -> Result<TraceId, E>
    where
        E: de::Error,
    {
        Ok(TraceId::Rendered(value.into()))
    }
}

impl ser::Serialize for TraceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> de::Deserialize<'de> for TraceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(TraceIdVisitor)
    }
}
