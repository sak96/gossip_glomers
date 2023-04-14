//! Implements echo node using [main].
use std::io::{stdin, stdout};

use gossip_glomers::{
    derive_request, derive_response,
    init::init,
    message::{Body, Message},
};

derive_request!(
    /// Request payload for echo node.
    pub enum EchoRequest {
        /// Echo request.
        ///
        /// ```json
        /// { "echo": "Please echo 35"}
        /// ```
        Echo {
            /// holds the message.
            echo: String,
        },
    }
);

derive_response!(
    /// Response payload for echo node.
    pub enum EchoResponse {
        /// Echo ok response.
        ///
        /// ```json
        /// {
        ///     "type": "echo_ok",
        ///     "echo": "Please echo 35"
        /// }
        /// ```
        EchoOk {
            /// holds the message.
            echo: String,
        },
    }
);

/// Echo node entry point.
///
/// The echo server
/// * Handle Initialization Protocol using [init].
/// * Read standard input for [Request][EchoRequest::Echo]
///   and reply with [Response][EchoResponse::EchoOk].
fn main() {
    let stdin = stdin().lock();
    let mut deseralizer = serde_json::Deserializer::from_reader(stdin);
    let mut stdout = stdout().lock();
    let _init = init(&mut stdout, &mut deseralizer);
    for (id, request) in deseralizer.into_iter::<Message<_>>().flatten().enumerate() {
        match request.body.payload {
            EchoRequest::Echo { echo } => Message {
                src: request.dst,
                dst: request.src,
                body: Body {
                    id: Some(id),
                    reply_id: request.body.id,
                    payload: EchoResponse::EchoOk { echo },
                },
            },
        }
        .send(&mut stdout);
    }
}
