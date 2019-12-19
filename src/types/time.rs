use serde::{de, ser, Serializer};
use std::{
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Represents fractional seconds since the epoch
/// These can be derived from std::time::Duration and be converted
/// to std::time::Duration
///
/// A Default implementation is provided which yields the number of seconds since the epoch from
/// the system time's `now` value
#[derive(Debug, PartialEq)]
pub struct Seconds(pub(crate) f64);

impl Seconds {
    /// return the current time in seconds since the unix epoch (1-1-1970 midnight)
    pub fn now() -> Self {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .into()
    }

    /// truncate epoc time to remove fractional seconds
    pub fn trunc(&self) -> u64 {
        self.0.trunc() as u64
    }
}

impl Default for Seconds {
    fn default() -> Self {
        Seconds::now()
    }
}

impl From<Duration> for Seconds {
    fn from(d: Duration) -> Self {
        Seconds(d.as_secs() as f64 + (f64::from(d.subsec_nanos()) / 1.0e9))
    }
}

impl Into<Duration> for Seconds {
    fn into(self) -> Duration {
        let Seconds(secs) = self;
        Duration::new(secs.trunc() as u64, (secs.fract() * 1.0e9) as u32)
    }
}

struct SecondsVisitor;

impl<'de> de::Visitor<'de> for SecondsVisitor {
    type Value = Seconds;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string value")
    }
    fn visit_f64<E>(self, value: f64) -> Result<Seconds, E>
    where
        E: de::Error,
    {
        Ok(Seconds(value))
    }
}

impl ser::Serialize for Seconds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Seconds(seconds) = self;
        serializer.serialize_f64(*seconds)
    }
}

impl<'de> de::Deserialize<'de> for Seconds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_f64(SecondsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Seconds;

    #[test]
    fn seconds_serialize() {
        assert_eq!(
            serde_json::to_string(&Seconds(1_545_136_342.711_932)).expect("failed to serialize"),
            "1545136342.711932"
        );
    }

    #[test]
    fn seconds_deserialize() {
        assert_eq!(
            serde_json::from_slice::<Seconds>(b"1545136342.711932").expect("failed to serialize"),
            Seconds(1_545_136_342.711_932)
        );
    }
}
