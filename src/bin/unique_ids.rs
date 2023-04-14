use std::io::{stdin, stdout};

use gossip_glomers::{
    derive_request, derive_response,
    init::{init, InitRequest},
    message::{Body, Message},
};

derive_request!(
    pub enum GenRequest {
        Generate,
    }
);

derive_response!(
    pub enum GenRespone {
        GenerateOk { id: String },
    }
);

fn main() {
    let stdin = stdin().lock();
    let mut deseralizer = serde_json::Deserializer::from_reader(stdin);
    let mut stdout = stdout().lock();
    let node_id = match init(&mut stdout, &mut deseralizer) {
        InitRequest::Init { node_id, .. } => node_id,
    };
    for (id, request) in deseralizer.into_iter::<Message<_>>().flatten().enumerate() {
        match request.body.payload {
            GenRequest::Generate => Message {
                src: request.dst,
                dst: request.src,
                body: Body {
                    id: Some(id),
                    reply_id: request.body.id,
                    payload: GenRespone::GenerateOk {
                        id: format!("{node_id}/{id}"),
                    },
                },
            },
        }
        .send(&mut stdout);
    }
}
