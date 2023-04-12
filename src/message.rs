use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
    pub fn recv<'a, R: serde_json::de::Read<'a>>(
        deseralizer: &mut serde_json::Deserializer<R>,
    ) -> Self {
        // TODO: fix this make the error not a issue.
        Self::deserialize(deseralizer).unwrap_or_else(|_| {
            panic!(
                "serialize response to{} failed",
                std::any::type_name::<Payload>(),
            )
        })
    }
}
