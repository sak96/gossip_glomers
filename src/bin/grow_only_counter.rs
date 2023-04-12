use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::{
    io::{stdin, stdout},
    sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use gossip_glomers::{
    init::{init, InitRequest},
    message::{Body, Message},
};
use serde::{Deserialize, Serialize};
// TODO: move these decoration to some macro.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CounterRequest {
    Add {
        delta: usize,
    },
    Read,
    Gossip {
        seen: Vec<(String, usize)>,
        you_saw: Vec<String>,
    },
}

// TODO: move these decoration to some macro.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CounterRespone {
    AddOk,
    ReadOk {
        value: usize,
    },
    Gossip {
        seen: Vec<(String, usize)>,
        you_saw: Vec<String>,
    },
}

pub enum Event {
    Tick,
    Close,
    Input(Message<CounterRequest>),
}

#[allow(dead_code)]
struct EventHandler {
    node: String,
    id: usize,
    value: usize,
    seen: HashSet<String>,
    known: Vec<(String, HashMap<String, usize>, HashSet<String>)>,
}

impl EventHandler {
    pub fn new(id: usize, init_request: InitRequest) -> Self {
        let (node, node_ids) = match init_request {
            InitRequest::Init { node_id, node_ids } => (node_id, node_ids),
        };
        Self {
            known: node_ids
                .into_iter()
                .map(|nid| (nid, HashMap::default(), HashSet::default()))
                .collect(),
            seen: HashSet::default(),
            value: 0,
            node,
            id,
        }
    }

    fn handle_input_payload(
        &mut self,
        payload: CounterRequest,
        src: &str,
        tick_tx: &mut Sender<()>,
    ) -> Option<CounterRespone> {
        match payload {
            CounterRequest::Add { delta } => {
                // FIXME: Use key store to store data.
                let msg_id = format!("{}{}", self.node, self.id);
                if self.seen.insert(msg_id.clone()) {
                    for (_, known, _) in self.known.iter_mut() {
                        known.insert(msg_id.clone(), delta);
                    }
                    self.value += delta;
                    tick_tx.send(()).expect("failed to tick");
                }
                Some(CounterRespone::AddOk)
            }
            CounterRequest::Read => Some(CounterRespone::ReadOk { value: self.value }),
            CounterRequest::Gossip { seen, you_saw } => {
                for (node, known, last_sent) in self.known.iter_mut() {
                    if node != src {
                        continue;
                    }
                    for msg_id in you_saw.iter() {
                        known.remove(msg_id);
                    }
                    let mut delta = 0;
                    last_sent.drain();
                    for (id, d) in seen.iter() {
                        if self.seen.insert(id.clone()) {
                            delta += d;
                        }
                        last_sent.insert(id.clone());
                    }
                    if delta > 0 {
                        self.value += delta;
                        tick_tx.send(()).expect("failed to tick");
                    }
                }
                None
            }
        }
    }
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
                    for (peer, known, last_sent) in self.known.iter_mut() {
                        let payload = match (
                            known.iter().map(|(x, y)| (x.clone(), *y)).collect::<Vec<_>>(),
                            last_sent.drain().collect::<Vec<_>>(),
                        ) {
                            (seen, you_saw) if seen.is_empty() & you_saw.is_empty() => continue,
                            (seen, you_saw) => CounterRespone::Gossip { seen, you_saw },
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

#[allow(unreachable_code, unused_variables)]
pub fn ticker(event_tx: Sender<Event>, tick_rx: Receiver<()>) {
    let duration = std::env::var("TICK_TIME")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(10);
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

fn main() {
    let mut stdout = stdout().lock();
    let id = 0;
    let init_request = {
        let stdin = stdin().lock();
        let mut deseralizer = serde_json::Deserializer::from_reader(stdin);
        init(&mut stdout, &mut deseralizer, Some(id))
    };
    let (event_tx, event_rx) = channel();
    let (tick_tx, tick_rx) = channel();
    std::thread::spawn({
        let event_tx = event_tx.clone();
        move || ticker(event_tx, tick_rx)
    });
    std::thread::spawn(move || input_recv(event_tx));
    EventHandler::new(id, init_request).handle_events(event_rx, tick_tx, &mut stdout);
}