use serde_repr::Deserialize_repr;

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
