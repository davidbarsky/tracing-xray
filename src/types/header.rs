//! X-Ray [tracing header](https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html?shortFooter=true#xray-concepts-tracingheader)
//! parser

use crate::types::ids::{SegmentId, TraceId};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
};

#[derive(PartialEq, Debug)]
pub enum SamplingDecision {
    /// Sampled indicates the current segment has been
    /// sampled and will be sent to the X-Ray daemon.
    Sampled,
    /// NotSampled indicates the current segment has
    /// not been sampled.
    NotSampled,
    ///sampling decision will be
    /// made by the downstream service and propagated
    /// back upstream in the response.
    Requested,
    /// Unknown indicates no sampling decision will be made.
    Unknown,
}

impl<'a> From<&'a str> for SamplingDecision {
    fn from(value: &'a str) -> Self {
        match value {
            "Sampled=1" => SamplingDecision::Sampled,
            "Sampled=0" => SamplingDecision::NotSampled,
            "Sampled=?" => SamplingDecision::Requested,
            _ => SamplingDecision::Unknown,
        }
    }
}

impl Display for SamplingDecision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SamplingDecision::Sampled => "Sampled=1",
                SamplingDecision::NotSampled => "Sampled=0",
                SamplingDecision::Requested => "Sampled=?",
                _ => "",
            }
        )?;
        Ok(())
    }
}

impl Default for SamplingDecision {
    fn default() -> Self {
        SamplingDecision::Unknown
    }
}

/// Parsed representation of `X-Amzn-Trace-Id` request header
#[derive(PartialEq, Debug, Default)]
pub struct Header {
    pub(crate) trace_id: TraceId,
    pub(crate) parent_id: Option<SegmentId>,
    pub(crate) sampling_decision: SamplingDecision,
    additional_data: HashMap<String, String>,
}

impl Header {
    /// HTTP header name associated with X-Ray trace data
    ///
    /// HTTP header values should be the Display serialization of Header structs
    pub const NAME: &'static str = "X-Amzn-Trace-Id";

    pub fn new(trace_id: TraceId) -> Self {
        Header {
            trace_id,
            ..Header::default()
        }
    }

    pub fn with_parent_id(&mut self, parent_id: SegmentId) -> &mut Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_sampling_decision(&mut self, decision: SamplingDecision) -> &mut Self {
        self.sampling_decision = decision;
        self
    }

    pub fn with_data<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.additional_data.insert(key.into(), value.into());
        self
    }
}

impl FromStr for Header {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(';')
            .try_fold(Header::default(), |mut header, line| {
                if line.starts_with("Root=") {
                    header.trace_id = TraceId::Rendered(line[5..].into())
                } else if line.starts_with("Parent=") {
                    header.parent_id = Some(SegmentId::Rendered(line[7..].into()))
                } else if line.starts_with("Sampled=") {
                    header.sampling_decision = line.into();
                } else if !line.starts_with("Self=") {
                    let pos = line
                        .find('=')
                        .ok_or_else(|| format!("invalid key=value: no `=` found in `{}`", s))?;
                    let (key, value) = (&line[..pos], &line[pos + 1..]);
                    header.additional_data.insert(key.into(), value.into());
                }
                Ok(header)
            })
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Root={}", self.trace_id)?;
        if let Some(parent) = &self.parent_id {
            write!(f, ";Parent={}", parent)?;
        }
        if self.sampling_decision != SamplingDecision::Unknown {
            write!(f, ";{}", self.sampling_decision)?;
        }
        for (k, v) in &self.additional_data {
            write!(f, ";{}={}", k, v)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_with_parent_from_str() {
        assert_eq!(
            "Root=1-5759e988-bd862e3fe1be46a994272793;Parent=53995c3f42cd8ad8;Sampled=1"
                .parse::<Header>(),
            Ok(Header {
                trace_id: TraceId::Rendered("1-5759e988-bd862e3fe1be46a994272793".into()),
                parent_id: Some(SegmentId::Rendered("53995c3f42cd8ad8".into())),
                sampling_decision: SamplingDecision::Sampled,
                ..Header::default()
            })
        )
    }
    #[test]
    fn parse_no_parent_from_str() {
        assert_eq!(
            "Root=1-5759e988-bd862e3fe1be46a994272793;Sampled=1".parse::<Header>(),
            Ok(Header {
                trace_id: TraceId::Rendered("1-5759e988-bd862e3fe1be46a994272793".into()),
                parent_id: None,
                sampling_decision: SamplingDecision::Sampled,
                ..Header::default()
            })
        )
    }

    #[test]
    fn displays_as_header() {
        let header = Header {
            trace_id: TraceId::Rendered("1-5759e988-bd862e3fe1be46a994272793".into()),
            ..Header::default()
        };
        assert_eq!(
            format!("{}", header),
            "Root=1-5759e988-bd862e3fe1be46a994272793"
        );
    }
}
