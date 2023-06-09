//! Message Protocol Implementation.
//!
//! Describe [Message] structure.
//! Provides function to send and receive message.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

/// Generic Message Structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct Message<Payload> {
    /// Source node name.
    pub src: String,
    #[serde(rename = "dest")]
    /// Destination node name.
    pub dst: String,
    /// Message Body.
    pub body: Body<Payload>,
}

/// Generic Message Body.
#[derive(Serialize, Deserialize, Debug)]
pub struct Body<Payload> {
    /// Message id.
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    /// Reply Message id.
    #[serde(rename = "in_reply_to")]
    pub reply_id: Option<usize>,
    /// Message Payload.
    #[serde(flatten)]
    pub payload: Payload,
}

/// Response trait to allow sending of messages.
pub trait Response {}

/// Request trait to allow receive of messages.
pub trait Request {}

impl<Payload: Serialize + Response> Message<Payload> {
    /// Sends serialized message by writing to writer.
    ///
    /// # Panics
    ///
    /// Panics if writing to writer fails.
    pub fn send<W: std::io::Write>(self, writer: &mut W) {
        serde_json::to_writer(&mut *writer, &self).unwrap_or_else(|_| {
            panic!(
                "serialize response to {} failed",
                std::any::type_name::<Payload>(),
            )
        });
        writer
            .write_all("\n".as_bytes())
            .expect("failed to send new line");
    }
}

impl<Payload: DeserializeOwned + Request> Message<Payload> {
    /// Receives de-serialized message by reading from reader.
    ///
    /// # Panics
    ///
    /// Panics if de-serialize or read fails.
    pub fn recv<'a, R: serde_json::de::Read<'a>>(
        deseralizer: &mut serde_json::Deserializer<R>,
    ) -> Self {
        Self::deserialize(deseralizer).unwrap_or_else(|_| {
            panic!(
                "serialize response to {} failed",
                std::any::type_name::<Payload>(),
            )
        })
    }
}

/// Derives trait for request `enum`.
///
/// The traits derived are:
/// * [::serde::Deserialize]
///     * uses tag as `type`.
///     * uses `snake_case` for de-serialize
/// * [Request]: allows receive Message with request payload.
/// * [Debug]
///
/// # Example
///
/// ```rust
/// # use gossip_glomers::derive_request;
/// # use gossip_glomers::message::Message;
/// derive_request!{
///   #[derive(PartialEq)]
///   pub enum PingRequest {
///     Ping
///   }
/// }
/// let input = r#"
///     {
///         "src": "src",
///         "dest": "dst",
///         "body": {
///             "msg_id": 1,
///             "type": "ping"
///         }
///     }
/// "#.as_bytes();
/// let mut deserializer = serde_json::Deserializer::from_reader(input);
/// let output = Message::<PingRequest>::recv(&mut deserializer);
/// assert_eq!(output.src, "src", "{:?}", output);
/// assert_eq!(output.dst, "dst", "{:?}", output);
/// assert_eq!(output.body.id, Some(1), "{:?}", output);
/// assert_eq!(output.body.reply_id, None, "{:?}", output);
/// assert_eq!(output.body.payload, PingRequest::Ping, "{:?}", output);
/// ```
#[macro_export]
macro_rules! derive_request {
    ($(#[$meta:meta])* $vis:vis enum $name:ident $body:tt ) => {
        $(#[$meta])*
        #[derive(::serde::Deserialize, Debug)]
        #[serde(tag = "type", rename_all = "snake_case")]
        $vis enum $name $body
        impl $crate::message::Request for $name {}
    };
}

/// Derives trait for response `enum`.
///
/// The traits derived are:
/// * [`::serde::Serialize`]
///     * uses tag as `type`.
///     * uses `snake_case` for serialize
/// * [Response]: allows send Message with response payload.
/// * [Debug]
///
/// # Example
///
/// ```rust
/// # use gossip_glomers::derive_response;
/// # use gossip_glomers::message::{Message, Body};
/// derive_response!{
///   #[derive(PartialEq)]
///   pub enum PingResponse {
///     Pong
///   }
/// }
/// let input = Message {
///     src: "src".to_string(),
///     dst: "dst".to_string(),
///     body: Body {
///         id: Some(1),
///         reply_id: Some(0),
///         payload: PingResponse::Pong,
///     }
/// };
/// let mut writer = Vec::new();
/// input.send(&mut writer);
/// let output = String::from_utf8_lossy(&writer);
/// let output = output.trim();
/// assert_eq!(output, r#"
///     {
///         "src": "src",
///         "dest": "dst",
///         "body": {
///             "msg_id": 1,
///             "in_reply_to": 0,
///             "type": "pong"
///         }
///     }
/// "#.chars().filter(|ch|!char::is_whitespace(*ch)).collect::<String>()
/// );
/// ```
#[macro_export]
macro_rules! derive_response {
    ($(#[$meta:meta])* $vis:vis enum $name:ident $body:tt ) => {
        $(#[$meta])*
        #[derive(::serde::Serialize, Debug)]
        #[serde(tag = "type", rename_all = "snake_case")]
        $vis enum $name $body
        impl $crate::message::Response for $name {}
    };
}

/// Error code when using services.
///
/// # Example
/// ```rust
/// pub enum Respone {
///    // .. Other Variation
///    Error { code: usize, text: String },
/// }
/// ```
#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum ErrorCode {
    /// Indicates that the requested operation could not be completed within a timeout.
    Timeout = 0,
    /// Thrown when a client sends an RPC request to a node which does not exist.
    NodeNotFound = 1,
    /// Use this error to indicate that a requested operation is not supported by the current implementation.
    /// Helpful for stubbing out APIs during development.
    NodeSupported = 10,
    /// Indicates that the operation definitely cannot be performed at this time.
    /// * perhaps because the server is in a read-only state
    /// * has not yet been initialized
    /// * believes its peers to be down
    /// * and so on.
    ///
    /// Do not use this error for indeterminate cases, when the operation may actually have taken place.
    TemporarilyUnavailable = 11,
    /// The client's request did not conform to the server's expectations, and could not possibly have been processed.
    MalformedRequest = 12,
    /// Indicates that some kind of general, indefinite error occurred.
    ///
    /// Use this as a catch-all for errors you can't otherwise categorize,
    /// or as a starting point for your error handler:
    /// it's safe to return internal-error for every problem by default,
    /// then add special cases for more specific errors later.
    Crash = 13,

    /// Indicates that some kind of general, definite error occurred.
    ///
    /// Use this as a catch-all for errors you can't otherwise categorize,
    /// when you specifically know that the requested operation has not taken place.
    /// For instance,
    /// you might encounter an indefinite failure during the prepare phase of a transaction:
    /// since you haven't started the commit process yet,
    /// the transaction can't have taken place.
    /// It's therefore safe to return a definite abort to the client.
    Abort = 14,
    /// The client requested an operation on a key which does not exist.
    ///
    /// Assuming the operation should not automatically create missing keys.
    KeyDoesNotExist = 20,
    /// The client requested the creation of a key which already exists, and the server will not overwrite it.
    KeyAlreadyExists = 21,
    /// The requested operation expected some conditions to hold, and those conditions were not met.
    ///
    /// For instance,
    /// a compare-and-set operation might assert that the value of a key is currently 5;
    /// if the value is 3, the server would return precondition-failed.
    PreconditionFailed = 22,
    /// The requested transaction has been aborted because of a conflict with another transaction.
    ///
    /// Servers need not return this error on every conflict: they may choose to retry automatically instead.
    TxnConflict = 30,
}
