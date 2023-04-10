use std::io::{stdin, stdout};

use gossip_glomers::{
    init::init,
    message::{Body, Message},
};
use serde::{Deserialize, Serialize};

// TODO: move these decoration to some macro.
#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EchoRequest {
    Echo { echo: String },
}

// TODO: move these decoration to some macro.
#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EchoRespone {
    EchoOk { echo: String },
}

fn main() {
    let stdin = stdin().lock();
    let mut deseralizer = serde_json::Deserializer::from_reader(stdin);
    let mut stdout = stdout().lock();
    let mut id = 0;
    let _init = init(&mut stdout, &mut deseralizer, Some(id));
    loop {
        id += 1;
        let echo_msg = Message::recv(&mut deseralizer);
        let response = match echo_msg.body.payload {
            EchoRequest::Echo { echo } => Message {
                src: echo_msg.dst,
                dst: echo_msg.src,
                body: Body {
                    id: Some(id),
                    reply_id: echo_msg.body.id,
                    payload: EchoRespone::EchoOk { echo },
                },
            },
        };
        response.send(&mut stdout);
    }
}
