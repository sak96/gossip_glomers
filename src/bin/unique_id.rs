use std::io::{stdin, stdout};

use gossip_glomers::{
    init::{init, InitRequest},
    message::{Body, Message},
};
use serde::{Deserialize, Serialize};

// TODO: move these decoration to some macro.
#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum GenRequest {
    Generate,
}

// TODO: move these decoration to some macro.
#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum GenRespone {
    GenerateOk { id: String },
}

fn main() {
    let stdin = stdin().lock();
    let mut deseralizer = serde_json::Deserializer::from_reader(stdin);
    let mut stdout = stdout().lock();
    let node_id = match init(&mut stdout, &mut deseralizer, Some(0)) {
        InitRequest::Init { node_id, .. } => node_id,
    };
    let mut id = 0;
    loop {
        let echo_msg = Message::recv(&mut deseralizer);
        let response = match echo_msg.body.payload {
            GenRequest::Generate => Message {
                src: echo_msg.dst,
                dst: echo_msg.src,
                body: Body {
                    id: Some(id),
                    reply_id: echo_msg.body.id,
                    payload: GenRespone::GenerateOk {
                        id: format!("{}/{}", node_id, id),
                    },
                },
            },
        };
        response.send(&mut stdout);
        id += 1;
    }
}
