use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    #[serde(rename = "in_reply_to")]
    pub reply_id: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

impl<Payload: Serialize> Message<Payload> {
    /// # Panics
    ///
    /// If the writing to writer fails!
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

impl<Payload: DeserializeOwned> Message<Payload> {
    /// # Panics
    ///
    /// If the de-serializer read fails!
    pub fn recv<'a, R: serde_json::de::Read<'a>>(
        deseralizer: &mut serde_json::Deserializer<R>,
    ) -> Self {
        Self::deserialize(deseralizer).unwrap_or_else(|_| {
            panic!(
                "serialize response to{} failed",
                std::any::type_name::<Payload>(),
            )
        })
    }
}

#[macro_export]
macro_rules! derive_request {
    ($vis:vis enum $name:ident $body:tt ) => {
        #[derive(::serde::Deserialize, Debug)]
        #[serde(tag = "type", rename_all = "snake_case")]
        $vis enum $name $body
    };
}

#[macro_export]
macro_rules! derive_response {
    ($vis:vis enum $name:ident $body:tt ) => {
        #[derive(::serde::Serialize, Debug)]
        #[serde(tag = "type", rename_all = "snake_case")]
        $vis enum $name $body
    };
}

/// Error code when using services.
///
/// ```rust,ignore
/// pub enum Respone {
///    ...
///    Error { code: usize, text: String },
/// }
/// ```
#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum ErrorCode {
    Timeout = 0,
    NodeNotFound = 1,
    NodeSupported = 10,
    TemporarilyUnavailable = 11,
    MalformedRequest = 12,
    Crash = 13,
    Abort = 14,
    KeyDoesNotExist = 20,
    KeyAlreadyExists = 21,
    PreconditionFailed = 22,
    TxnConflict = 30,
}
