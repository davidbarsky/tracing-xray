use super::{
    ids::{SegmentId, TraceId},
    time::Seconds,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::ops::Not;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Segment {
    /// A unique identifier that connects all segments and subsegments
    /// originating from a single client request.
    pub(crate) trace_id: TraceId,
    ///  A 64-bit identifier for the segment, unique among segments in the same
    ///  trace, in 16 hexadecimal digits.
    pub(crate) id: SegmentId,
    /// The logical name of the service that handled the request, up to 200
    /// characters. For example, your application's name or domain name. Names
    /// can contain Unicode letters, numbers, and whitespace, and the following
    /// symbols: `_`, `.`,`:`, `/`, `%`, `&, `#`, `=`, `+`, `\`, `-`, `@`
    ///
    /// A segment's name should match the domain name or logical name of the
    /// service that generates the segment. However, this is not enforced. Any
    /// application that has permission to PutTraceSegments can send segments
    /// with any name.
    pub(crate) name: String,
    /// Number that is the time the segment was created, in floating point
    /// seconds in epoch time.
    pub(crate) start_time: Seconds,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Number that is the time the segment was closed.
    pub end_time: Option<Seconds>,
    #[serde(skip_serializing_if = "Not::not")]
    ///  boolean, set to true instead of specifying an end_time to record that a
    ///  segment is started, but is not complete. Send an in-progress segment
    ///  when your application receives a request that will take a long time to
    ///  serve, to trace the request receipt. When the response is sent, send
    ///  the complete segment to overwrite the in-progress segment. Only send
    ///  one complete segment, and one or zero in-progress segments, per
    ///  request.
    pub in_progress: bool,
    /// A subsegment ID you specify if the request originated from an
    /// instrumented application. The X-Ray SDK adds the parent subsegment ID to
    /// the tracing header for downstream HTTP calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<SegmentId>,
    /// Indicates that a server error occurred (response status code was 5XX
    /// Server Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub fault: bool,
    /// Indicates that a client error occurred (response status code was 4XX
    /// Client Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub error: bool,
    /// boolean indicating that a request was throttled (response status code
    /// was 429 Too Many Requests).
    #[serde(skip_serializing_if = "Not::not")]
    pub throttle: bool,
    ///  error fields that indicate an error occurred and that include
    ///  information about the exception that caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
    /// The type of AWS resource running your application. todo: convert to enum
    /// (see aws docs for values) When multiple values are applicable to your
    /// application, use the one that is most specific. For example, a
    /// Multicontainer Docker Elastic Beanstalk environment runs your
    /// application on an Amazon ECS container, which in turn runs on an Amazon
    /// EC2 instance. In this case you would set the origin to
    /// AWS::ElasticBeanstalk::Environment as the environment is the parent of
    /// the other two resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// A string that identifies the user who sent the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_arn: Option<String>,
    /// http objects with information about the original HTTP request.
    /// #[serde(skip_serializing_if = "Option::is_none")] pub http:
    /// Option<Http>, annotations object with key-value pairs that you want
    /// X-Ray to index for search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, Annotation>>,
    /// metadata object with any additional data that you want to store in the
    /// segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
    /// aws object with information about the AWS resource on which your
    /// application served the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws: Option<Aws>,
    /// An object with information about your application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
}

impl Segment {
    /// Begins a new named segment
    ///
    /// A segment's name should match the domain name or logical name of the service that generates the segment. However, this is not enforced. Any application that has permission to PutTraceSegments can send segments with any name.
    pub fn begin<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        let mut valid_name = name.into();
        if valid_name.len() > 200 {
            valid_name = valid_name[..200].into();
        }
        Segment {
            name: valid_name,
            ..Segment::default()
        }
    }

    /// End the segment by assigning its end_time
    pub fn end(&mut self) -> &mut Self {
        self.end_time = Some(Seconds::now());
        self.in_progress = false;
        self
    }
}

/// A value type which may be used for
/// filter querying
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Annotation {
    /// A string value
    String(String),
    /// A numberic value
    Number(usize),
    /// A boolean value
    Bool(bool),
}

impl Default for Annotation {
    fn default() -> Self {
        Annotation::String("".into())
    }
}

/// Describes an http request/response cycle
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Http {
    /// Information about a request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    /// Information about a response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Response>,
}

///  Information about a request.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Request {
    /// The request method. For example, GET.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// The full URL of the request, compiled from the protocol, hostname, and path of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The IP address of the requester. Can be retrieved from the IP packet's Source Address or, for forwarded requests, from an X-Forwarded-For header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    /// The user agent string from the requester's client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// (segments only) boolean indicating that the client_ip was read from an X-Forwarded-For header and is not reliable as it could have been forged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_forwarded_for: Option<String>,
    /// (subsegments only) boolean indicating that the downstream call is to another traced service. If this field is set to true, X-Ray considers the trace to be broken until the downstream service uploads a segment with a parent_id that matches the id of the subsegment that contains this block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traced: Option<bool>,
}

///  Information about a response.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response {
    /// number indicating the HTTP status of the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// number indicating the length of the response body in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_length: Option<u64>,
}

///  An object with information about your application.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Service {
    /// A string that identifies the version of your application that served the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Context information about the AWS environment this segment was run in
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Aws {
    ///  If your application sends segments to a different AWS account, record the ID of the account running your application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    ///  Information about an Amazon ECS container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecs: Option<Ecs>,
    ///  Information about an EC2 instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ec2: Option<Ec2>,
    /// Information about an Elastic Beanstalk environment. You can find this information in a file named /var/elasticbeanstalk/xray/environment.conf on the latest Elastic Beanstalk platforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elastic_beanstalk: Option<ElasticBeanstalk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<Tracing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xray: Option<XRay>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct XRay {
    pub sdk_version: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ecs {
    /// The container ID of the container running your application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ec2 {
    /// The instance ID of the EC2 instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The Availability Zone in which the instance is running.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_zone: Option<String>,
}

/// Information about an Elastic Beanstalk environment. You can find this information in a file named /var/elasticbeanstalk/xray/environment.conf on the latest Elastic Beanstalk platforms.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ElasticBeanstalk {
    /// The name of the environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_name: Option<String>,
    ///  The name of the application version that is currently deployed to the instance that served the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_label: Option<String>,
    /// number indicating the ID of the last successful deployment to the instance that served the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<usize>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Tracing {
    /// version of sdk
    pub sdk: String,
}

/// Detailed representation of an exception
#[derive(Debug, Serialize, Deserialize)]
pub struct Exception {
    /// A 64-bit identifier for the exception, unique among segments in the same trace, in 16 hexadecimal digits.
    pub id: String,
    /// The exception message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<String>,
    /// The exception type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<bool>,
    /// integer indicating the number of stack frames that are omitted from the stack.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<usize>,
    ///  integer indicating the number of exceptions that were skipped between this exception and its child, that is, the exception that it caused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<usize>,
    /// Exception ID of the exception's parent, that is, the exception that caused this exception.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// array of stackFrame objects.
    pub stack: Vec<StackFrame>,
}

/// A summary of a single operation within a stack trace
#[derive(Debug, Serialize, Deserialize)]
pub struct StackFrame {
    /// The relative path to the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The line in the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    /// The function or method name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Represents the cause of an errror
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Cause {
    ///  a 16 character exception ID
    Name(String),
    /// A description of an error
    Description {
        ///  The full path of the working directory when the exception occurred.
        working_directory: String,
        ///  The array of paths to libraries or modules in use when the exception occurred.
        paths: Vec<String>,
        /// The array of exception objects.
        exceptions: Vec<Exception>,
    },
}

/// Wraps a byte slice to enable lowcast hex display formatting
pub(crate) struct Bytes<'a>(pub(crate) &'a [u8]);

impl fmt::LowerHex for Bytes<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            fmt.write_fmt(format_args!("{:02x}", byte))?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Bytes;
    #[test]
    fn formats_lowerhex() {
        assert_eq!(format!("{:x}", Bytes(b"test")), "74657374")
    }
}
