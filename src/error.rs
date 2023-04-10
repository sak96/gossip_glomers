use serde::Deserialize;

#[derive(Deserialize)]
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

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ErrorRespone {
    Error { code: usize, text: String },
}
