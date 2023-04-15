//! Implements unique id generation node using [main].
use std::io::{stdin, stdout};

use gossip_glomers::{
    derive_request, derive_response,
    init::{init, InitRequest},
    message::{Body, Message},
};

derive_request!(
    /// Request payload for unique id generation node.
    pub enum GenRequest {
        /// Generate Id request.
        ///
        /// ```json
        /// { "type": "generate" }
        /// ```
        Generate,
    }
);

derive_response!(
    /// Response payload for unique id generation node.
    pub enum GenRespone {
        /// Generate ok response.
        ///
        /// ```json
        /// {
        ///     "type": "generate_ok",
        ///     "id": 123
        /// }
        /// ```
        GenerateOk {
            /// Newly generated id.
            id: String,
        },
    }
);

/// Unique Id generation node entry point.
///
/// The unique id server.
/// * Handle Initialization Protocol using [init].
/// * Read standard input for [Request][GenRequest::Generate]
///   and reply with [Response][GenRespone::GenerateOk].
///
/// # Logic
///
/// Id is in format `node id` and `counter` separated by `/`.
/// The node id guarantees that the id generated by two node don't conflict.
/// The counter is incremented after id generation.
/// This guarantees uniqueness for id generated from same node.
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
