//! Implements grow counter node using [main].
use std::{
    io::{stdin, stdout},
    sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use gossip_glomers::{
    derive_request, derive_response,
    init::{init, InitRequest},
    message::{Body, ErrorCode, Message},
};

derive_request!(
    /// Request payload for counter node.
    pub enum CounterRequest {
        /// Add request.
        ///
        /// This message requests that a value be incremented to a single global counter.
        /// ```json
        /// {
        ///     "type": "add",
        ///     "delta": 10
        /// }
        /// ```
        Add {
            /// delta value.
            delta: usize
        },
        /// Read request.
        ///
        /// This message requests that value be read.
        /// ```json
        /// { "type": "read" }
        /// ```
        Read,
        /// Counter value request.
        ///
        /// This message acknowledge [CounterResponse::ReadCounter].
        /// ```json
        /// {
        ///     "type": "read_ok",
        ///     "value": 10
        /// }
        /// ```
        #[serde(rename = "read_ok")]
        ReadCounterOk { 
            /// counter value.
            value: usize
        },
        /// Update Success request.
        ///
        /// This message acknowledge [CounterResponse::UpdateCounter].
        /// ```json
        /// { "type": "cas_ok" }
        /// ```
        #[serde(rename = "cas_ok")]
        CounterUpdated,
        /// Error request.
        ///
        /// This message acknowledge error in operation.
        /// ```json
        /// {
        ///     "type": "error"
        ///     "code": 20,
        ///     "text": "Some messsage"
        /// }
        /// ```
        Error {
            /// error code.
            code: ErrorCode,
            /// error message.
            text: String
        },
    }
);

derive_response!(
    /// Response payload for counter node.
    pub enum CounterResponse {
        /// Add ok response.
        ///
        /// This message acknowledge to [CounterRequest::Add].
        /// ```json
        /// { "type": "add_ok" }
        /// ```
        AddOk,
        /// Read counter response.
        ///
        /// This message to read to counter value from key store.
        /// ```json
        /// {
        ///     "type": "read",
        ///     "key": "COUNTER"
        /// }
        /// ```
        #[serde(rename = "read")]
        ReadCounter {
            /// Key of counter from key store.
            key: String,
        },
        /// Update counter response.
        ///
        /// This message to update to counter value in key store.
        /// ```json
        /// {
        ///     "type": "cas",
        ///     "key": "COUNTER",
        ///     "old": 10,
        ///     "new": 20
        /// }
        /// ```
        #[serde(rename = "cas")]
        UpdateCounter {
            /// Counter key in store.
            key: String,
            /// Value to be updated from.
            #[serde(rename = "from")]
            old: usize,
            /// Value to be updated to.
            #[serde(rename = "to")]
            new: usize,
            /// Create key if not exists.
            #[serde(rename = "create_if_not_exists")]
            create: bool,
        },
        /// Read Ok response.
        ///
        /// This message acknowledge to [CounterResponse::ReadOk].
        /// It provide counter value from memory.
        /// ```json
        /// {
        ///     "type": "read_ok",
        ///     "value": 20
        /// }
        /// ```
        ReadOk {
            /// The value of counter from memory.
            value: usize,
        },
    }
);

/// Node id for key store.
const KV_NODE: &str = "seq-kv";
/// Key of the counter from store.
const KEY: &str = "COUNTER";

/// Event for node to handle.
pub enum Event {
    /// Tick Event to handle timer based events.
    Tick,
    /// Close Event to stop node.
    Close,
    /// Input Event from other nodes.
    Input(Message<CounterRequest>),
}

/// Event handler for grow counter node.
struct EventHandler {
    /// Message response id counter.
    id: usize,
    /// Node id.
    node: String,
    /// Value of counter.
    value: usize,
    /// Delta for counter.
    delta: usize,
    /// Counter update status.
    ///
    /// Stores:
    ///     - update counter message id,
    ///     - old counter value.
    ///     - new counter value.
    last_update: Option<(usize, usize, usize)>,
}

impl EventHandler {
    /// Create new event handler from initialization message.
    pub fn new(init_request: InitRequest) -> Self {
        Self {
            id: 0,
            value: 0,
            delta: 0,
            node: match init_request {
                InitRequest::Init { node_id, .. } => node_id,
            },
            last_update: None,
        }
    }
    /// Handle input requests.
    ///
    /// Handle requests in following ways:
    /// * [Add](CounterRequest::Add):
    ///     * add to delta and send add ok.
    /// * [Read](CounterRequest::Read):
    ///     * send force tick.
    ///     * send read ok with current value + delta.
    /// * [Read counter ok](CounterRequest::ReadCounterOk):
    ///     * update current value to new value + delta.
    ///     * if delta > 0 then
    ///         * set counter update delta.
    ///         * send update counter request.
    /// * [Counter update](CounterRequest::CounterUpdated):
    ///     * unset counter update delta.
    /// * [Error](CounterRequest::Error):
    ///     * [KeyDoesNotExist](ErrorCode::KeyDoesNotExist):
    ///         * update counter failed due to key not existing.
    ///         * send create key request.
    ///     * Update key failed with errors:
    ///         * [precondition failed](ErrorCode::PreconditionFailed)
    ///         * [timeout](ErrorCode::Timeout)
    ///         * [key already exists](ErrorCode::KeyAlreadyExists)
    ///         * re-send previous update request.
    ///
    /// # Arguments
    /// * payload: request to be handled requests.
    /// * src: source node id.
    /// * tick_tx: tick sender to allow force ticking.
    ///
    /// # Returns
    /// Response if any for payload.
    fn handle_input_payload(
        &mut self,
        payload: CounterRequest,
        _src: &str,
        tick_tx: &mut Sender<()>,
    ) -> Option<CounterResponse> {
        match payload {
            CounterRequest::Add { delta } => {
                self.delta += delta;
                Some(CounterResponse::AddOk)
            }
            CounterRequest::Read => {
                tick_tx.send(()).expect("force ticking failed");
                Some(CounterResponse::ReadOk {
                    value: self.value + self.delta,
                })
            }
            CounterRequest::ReadCounterOk { value } => {
                self.value = value + self.delta;
                if self.delta > 0 {
                    self.last_update = Some((self.id, value, self.value));
                    Some(CounterResponse::UpdateCounter {
                        key: KEY.into(),
                        old: value,
                        new: value + std::mem::take(&mut self.delta),
                        create: false,
                    })
                } else {
                    None
                }
            }
            CounterRequest::CounterUpdated => {
                self.last_update.take();
                None
            }
            CounterRequest::Error { code, .. } => {
                if let Some((_, old, new)) = self.last_update.take() {
                    self.delta += new - old;
                    tick_tx.send(()).expect("force ticking failed");
                };
                match code {
                    ErrorCode::KeyDoesNotExist => Some(CounterResponse::UpdateCounter {
                        key: KEY.into(),
                        old: 0,
                        new: 0,
                        create: true,
                    }),
                    ErrorCode::PreconditionFailed
                    | ErrorCode::Timeout
                    | ErrorCode::KeyAlreadyExists => None,
                    error => panic!("Unhandled error code: {error:?}"),
                }
            }
        }
    }
    /// Handle events.
    ///
    /// Handle events in following ways:
    /// * [close](Event::Close): close the loop.
    /// * [tick](Event::Tick):
    ///     * send [CounterResponse::UpdateCounter] if there is some delta.
    ///     * else send [CounterResponse::ReadCounter] if there is no delta.
    /// * [input](Event::Input):
    ///     * send payload to [Self::handle_input_payload].
    ///     * send any response via writer.
    ///
    /// # Arguments
    /// * rx: Events receiver Channel.
    /// * tick_tx: Tick sender to allow force ticking.
    /// * writer: Output response via writer.
    pub fn handle_events<W: std::io::Write>(
        &mut self,
        rx: Receiver<Event>,
        mut tick_tx: Sender<()>,
        writer: &mut W,
    ) {
        for event in rx.iter() {
            match event {
                Event::Close => {
                    break;
                }
                Event::Tick => {
                    let key = KEY.into();
                    let (payload, msg_id) = if let Some((msg_id, old, new)) = self.last_update {
                        (
                            CounterResponse::UpdateCounter {
                                key,
                                old,
                                new,
                                create: false,
                            },
                            msg_id,
                        )
                    } else {
                        let id = self.id;
                        self.id += 1;
                        (CounterResponse::ReadCounter { key }, id)
                    };
                    let response = Message {
                        body: Body {
                            id: Some(msg_id),
                            reply_id: None,
                            payload,
                        },
                        src: self.node.clone(),
                        dst: KV_NODE.into(),
                    };
                    response.send(writer);
                }
                Event::Input(request) => {
                    if let Some(payload) =
                        self.handle_input_payload(request.body.payload, &request.src, &mut tick_tx)
                    {
                        let response = Message {
                            body: Body {
                                id: Some(self.id),
                                reply_id: request.body.id,
                                payload,
                            },
                            src: request.dst,
                            dst: request.src,
                        };
                        response.send(writer);
                        self.id += 1;
                    }
                }
            };
        }
    }
}

/// Send tick event to node and provides force ticking.
pub fn ticker(event_tx: Sender<Event>, tick_rx: Receiver<()>) {
    let duration = std::env::var("TICK_TIME")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(300);
    while matches!(
        tick_rx.recv_timeout(Duration::from_millis(duration)),
        Err(RecvTimeoutError::Timeout) | Ok(_)
    ) {
        tick_rx.try_iter().fuse().for_each(drop);
        event_tx
            .send(Event::Tick)
            .expect("Message should be passed!");
    }
}

/// Receive input and send events to channel.
pub fn input_recv(event_tx: Sender<Event>) {
    let stdin = stdin().lock();
    let deseralizer = serde_json::Deserializer::from_reader(stdin);
    for input_request in deseralizer.into_iter().flatten() {
        if event_tx.send(Event::Input(input_request)).is_err() {
            break;
        }
    }
    event_tx.send(Event::Close).expect("failed to close");
}

/// Grow counter node entry point.
///
/// The grow counter server
/// * Handle Initialization Protocol using [init].
/// * Spawn [ticker] thread.
/// * Spawn [input_recv] thread.
/// * Run [EventHandler::handle_events].
///
/// # Consensus Logic
///
/// * Node keeps track of delta and value.
/// * On tick:
///     * if there is pending update resend with same message id.
///     * else read counter value.
/// * On receiving counter value.
///     * update value = read value + delta.
///     * if delta > 0
///         * send update counter request (compare and swap).
///             * with previous value as read value.
///             * with new value as value (read value + delta).
///         * store message details for re-sending on error.
/// * On error which only matters for compare and swap failure.
///     * revert last update status back to delta.
///         * delta = delta + new value - old value.
///     * then
///         * if error is due to key not existing, create key (compare and swap),
///           with previous value and new value as 0.
///         * if error is due to compare swap condition failure or time out
///           or key already exits, then back off wait for next tick.
///         * other errors are unhandled.
fn main() {
    let mut stdout = stdout().lock();
    let init_request = {
        let stdin = stdin().lock();
        let mut deseralizer = serde_json::Deserializer::from_reader(stdin);
        init(&mut stdout, &mut deseralizer)
    };
    let (event_tx, event_rx) = channel();
    let (tick_tx, tick_rx) = channel();
    std::thread::spawn({
        let event_tx = event_tx.clone();
        move || ticker(event_tx, tick_rx)
    });
    std::thread::spawn(move || input_recv(event_tx));
    EventHandler::new(init_request).handle_events(event_rx, tick_tx, &mut stdout);
}
