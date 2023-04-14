use crate::{
    derive_request, derive_response,
    message::{Body, Message},
};

derive_request!(
    pub enum InitRequest {
        Init {
            node_id: String,
            node_ids: Vec<String>,
        },
    }
);

derive_response!(
    pub enum InitRespone {
        InitOk,
    }
);

pub fn init<'a, W: std::io::Write, R: serde_json::de::Read<'a>>(
    writer: &mut W,
    deseralizer: &mut serde_json::Deserializer<R>,
) -> InitRequest {
    let init_msg = Message::recv(deseralizer);
    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id: None,
            reply_id: init_msg.body.id,
            payload: InitRespone::InitOk,
        },
    };
    reply.send(writer);
    init_msg.body.payload
}
