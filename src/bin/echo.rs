use std::io::{stdin, stdout};

use gossip_glomers::{
    init::init,
    message::{Body, Message},
};
use serde::{Deserialize, Serialize};

// TODO: move these decoration to some macro.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoRequest {
    Echo { echo: String },
}

// TODO: move these decoration to some macro.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoRespone {
    EchoOk { echo: String },
}

fn main() {
    let stdin = stdin().lock();
    let mut deseralizer = serde_json::Deserializer::from_reader(stdin);
    let mut stdout = stdout().lock();
    let _init = init(&mut stdout, &mut deseralizer, None);
    for (id, request) in deseralizer.into_iter::<Message<_>>().flatten().enumerate() {
        let response = match request.body.payload {
            EchoRequest::Echo { echo } => Message {
                src: request.dst,
                dst: request.src,
                body: Body {
                    id: Some(id),
                    reply_id: request.body.id,
                    payload: EchoRespone::EchoOk { echo },
                },
            },
        };
        response.send(&mut stdout);
    }
}
