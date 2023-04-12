use serde::{Deserialize, Serialize};


use crate::message::{Body, Message};

// TODO: move these decoration to some macro.
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitRequest {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
}

// TODO: move these decoration to some macro.
#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitRespone {
    InitOk,
}

pub fn init<'a, W: std::io::Write, R: serde_json::de::Read<'a>>(
    writer: &mut W,
    deseralizer: &mut serde_json::Deserializer<R>,
    id: Option<usize>,
) -> InitRequest {
    let init_msg = Message::recv(deseralizer);
    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id,
            reply_id: init_msg.body.id,
            payload: InitRespone::InitOk,
        },
    };
    reply.send(writer);
    init_msg.body.payload
}
