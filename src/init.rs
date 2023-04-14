//! Initialization Protocol Implementation.
use crate::{
    derive_request, derive_response,
    message::{Body, Message},
};

derive_request!(
    /// Initialization Request
    ///
    /// Maelstrom issues a single `init` message to each node.
    ///
    /// # Examples
    /// Note that the `msg_id` is part of body.
    /// ```json
    /// {
    ///     "msg_id": 1,
    ///     "type": "init",
    ///     "node_id": "n3",
    ///     "node_ids": ["n1", "n2", "n3"]
    /// }
    /// ```
    pub enum InitRequest {
        /// Initialization Request Payload.
        Init {
            /// ID of the node which is receiving this message.
            ///
            /// Include ID as the `src` of any message the node sends.
            node_id: String,
            /// Lists of all nodes ID in the cluster, including the recipient.
            node_ids: Vec<String>,
        },
    }
);

derive_response!(
    /// Initialization Request
    ///
    /// Response to the `init` message.
    ///
    /// # Examples
    /// Note that the `in_reply_to` is part of body.
    /// ```json
    /// {
    ///     "in_reply_to": 1
    ///     "type": "init_ok",
    /// }
    /// ```
    pub enum InitRespone {
        /// Initialization Response Payload.
        InitOk,
    }
);

/// Handles Initialization Protocol and returns Initialization payload.
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
