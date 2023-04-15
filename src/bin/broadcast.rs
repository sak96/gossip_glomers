//! Implements broadcast node using [main].
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::{
    io::{stdin, stdout},
    sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use gossip_glomers::{
    derive_request, derive_response,
    init::{init, InitRequest},
    message::{Body, Message},
};

derive_request!(
    /// Request payload for broadcast node.
    pub enum BroadcastRequest {
        /// Broadcast request.
        ///
        /// This message requests that a value be broadcast out to all nodes in the cluster.
        /// ```json
        /// {
        ///     "type": "broadcast",
        ///     "message": 1000
        /// }
        /// ```
        Broadcast {
            /// message to broadcast.
            message: usize,
        },
        /// Read request.
        ///
        /// This message requests that a node return all values that it has seen.
        /// ```json
        /// { "type": "read" }
        /// ```
        Read,
        /// Topology request.
        ///
        /// This message informs the node of who its neighboring nodes are.
        /// ```json
        /// {
        ///     "type": "topology",
        ///     "topology": {
        ///         "n1": ["n2", "n3"],
        ///         "n2": ["n1"],
        ///         "n3": ["n1"]
        ///     }
        /// }
        /// ```
        Topology {
            /// Map from node to all the its neighboring nodes.
            topology: HashMap<String, Vec<String>>,
        },
        /// Consensus request.
        ///
        /// This message informs new values seen from other nodes.
        /// It includes values newly seen by the other node.
        /// It also acknowledges last response by current to other node.
        /// ```json
        /// {
        ///     "type": "consensus",
        ///     "seen": ["2", "3"],
        ///     "seen_ack": ["2", "3"]
        /// }
        /// ```
        Consensus {
            /// Values seen newly by other node.
            seen: HashSet<usize>,
            /// Values received in last request of current node.
            seen_ack: Vec<usize>,
        },
    }
);

derive_response!(
    /// Response payload for broadcast node.
    pub enum BroadcastRespone {
        /// Broadcast ok response.
        ///
        /// This message acknowledges Broadcast request.
        /// ```json
        /// { "type": "broadcast_ok" }
        /// ```
        BroadcastOk,
        /// Read ok response.
        ///
        /// This message acknowledges Read request.
        /// It includes a list of values it has seen.
        /// ```json
        /// {
        ///     "type": "read_ok",
        ///     "messages": [1, 8, 72, 25]
        /// }
        /// ```
        ReadOk {
            /// List of all message seen until now.
            messages: HashSet<usize>,
        },
        /// Topology ok response.
        ///
        /// This message acknowledges Topology request.
        /// ```json
        /// { "type": "topology_ok" }
        /// ```
        TopologyOk,
        /// Consensus response.
        ///
        /// This message informs new values seen by current nodes.
        /// It also acknowledges last request by other to current node.
        /// ```json
        /// {
        ///     "type": "consensus",
        ///     "seen": ["2", "3"],
        ///     "seen_ack": ["2", "3"]
        /// }
        /// ```
        Consensus {
            /// Values seen newly by current node.
            seen: Vec<usize>,
            /// Values received in last response of other node.
            seen_ack: Vec<usize>,
        },
    }
);

/// Event for node to handle.
pub enum Event {
    /// Tick Event to handle timer based events.
    Tick,
    /// Close Event to stop node.
    Close,
    /// Input Event from other nodes.
    Input(Message<BroadcastRequest>),
}

/// Event handler for broadcast node.
struct EventHandler {
    /// Message response id counter.
    id: usize,
    /// Node id.
    node: String,
    /// Message seen till now.
    messages: HashSet<usize>,
    /// Memory of other nodes seen message.
    ///
    /// Known map from other node id to known id and last seen nodes.
    known: HashMap<String, (HashSet<usize>, HashSet<usize>)>,
    /// Peer of current node.
    peers: HashSet<String>,
    /// Force tick.
    force: bool,
}

impl EventHandler {
    /// Create new event handler from initialization message.
    pub fn new(init_request: InitRequest) -> Self {
        let (node, node_ids) = match init_request {
            InitRequest::Init { node_id, node_ids } => (node_id, node_ids),
        };
        let force = std::env::var("FORCE_TICK")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or(true);
        Self {
            id: 0,
            known: node_ids
                .into_iter()
                .filter(|n| !n.eq(&node))
                .map(|nid| (nid, (HashSet::default(), HashSet::default())))
                .collect(),
            messages: HashSet::default(),
            peers: HashSet::default(),
            node,
            force,
        }
    }
    /// Handle input requests.
    ///
    /// Handle requests in following ways:
    /// * [Broadcast](BroadcastRequest::Broadcast):
    ///     * remember the message and force tick.
    ///     * are replied with broadcast ok.
    /// * [Read](BroadcastRequest::Read):
    ///     * send read ok with all messages.
    /// * [Topology](BroadcastRequest::Topology):
    ///     * update peers list.
    /// * [Consensus](BroadcastRequest::Consensus):
    ///     * For any new message update seen and force tick.
    ///     * Update the source node's known list.
    ///     * Remember the message for seen_ack.
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
        payload: BroadcastRequest,
        src: &str,
        tick_tx: &mut Sender<()>,
    ) -> Option<BroadcastRespone> {
        match payload {
            BroadcastRequest::Broadcast { message } => {
                if self.messages.insert(message) & self.force {
                    tick_tx.send(()).expect("failed to tick");
                }
                Some(BroadcastRespone::BroadcastOk)
            }
            BroadcastRequest::Read => Some(BroadcastRespone::ReadOk {
                messages: self.messages.clone(),
            }),
            BroadcastRequest::Topology { mut topology } => {
                if let Some(peers) = topology.remove(&self.node) {
                    self.peers = peers.into_iter().collect();
                }
                Some(BroadcastRespone::TopologyOk)
            }
            BroadcastRequest::Consensus { seen, seen_ack } => {
                let (known, last_sent) = self.known.get_mut(src).expect("node are pre-determined");
                known.extend(seen_ack.iter());
                if !self.messages.is_superset(&seen) {
                    self.messages.extend(seen.iter().copied());
                    if self.force {
                        tick_tx.send(()).expect("failed to tick");
                    }
                }
                *last_sent = seen;
                None
            }
        }
    }
    /// Handle events.
    ///
    /// Handle events in following ways:
    /// * [close](Event::Close): close the loop.
    /// * [tick](Event::Tick):
    ///     * send [Consensus](BroadcastRequest::Consensus) message to every peer.
    ///     * send only difference from known of peer and message list.
    ///     * send acknowledge for  peers last [Consensus](BroadcastRequest::Consensus).
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
                    for peer in &self.peers {
                        let (known, last_sent) =
                            self.known.get_mut(peer).expect("node are pre-determined");
                        let payload = match (
                            self.messages.difference(known).copied().collect::<Vec<_>>(),
                            last_sent.drain().collect::<Vec<_>>(),
                        ) {
                            (seen, seen_ack) if seen.is_empty() & seen_ack.is_empty() => continue,
                            (seen, seen_ack) => BroadcastRespone::Consensus { seen, seen_ack },
                        };
                        let response = Message {
                            body: Body {
                                id: None,
                                reply_id: None,
                                payload,
                            },
                            src: self.node.to_string(),
                            dst: peer.to_string(),
                        };
                        response.send(writer);
                        self.id += 1;
                    }
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
        .unwrap_or(200);
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

/// Broadcast node entry point.
///
/// The broadcast server
/// * Handle Initialization Protocol using [init].
/// * Spawn [ticker] thread.
/// * Spawn [input_recv] thread.
/// * Run [EventHandler::handle_events].
///
/// # Consensus Logic
/// * Current node keeps track of all other nodes know list.
/// * On every tick it sends consensus message to peers.
/// * The consensus will be reached when
///     * Current node sends new item in [Consensus](BroadcastRequest::Consensus) for peer.
///     * Peer then send [Consensus](BroadcastRequest::Consensus) with seen_ack containing the new item.
///     * If a seen_ack is not received between tick then the new item is sent again.
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
