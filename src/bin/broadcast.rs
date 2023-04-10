use hashbrown::{HashMap, HashSet};
use std::{
    io::{stdin, stdout},
    sync::mpsc::{channel, sync_channel, Receiver, RecvTimeoutError, Sender, SyncSender},
    time::Duration,
};

use gossip_glomers::{
    init::{init, InitRequest},
    message::{Body, Message},
};
use serde::{Deserialize, Serialize};
// TODO: move these decoration to some macro.
#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BroadCastRequest {
    Broadcast {
        message: usize,
    },
    Read,
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    Gossip {
        seen: HashSet<usize>,
        you_saw: HashSet<usize>,
    },
}

// TODO: move these decoration to some macro.
#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BroadCastRespone {
    BroadcastOk,
    ReadOk {
        messages: HashSet<usize>,
    },
    TopologyOk,
    Gossip {
        seen: HashSet<usize>,
        you_saw: HashSet<usize>,
    },
}

pub enum Event {
    Tick,
    Input(Message<BroadCastRequest>),
}

#[allow(dead_code)]
struct EventHandler {
    node: String,
    id: usize,
    messages: HashSet<usize>,
    known: HashMap<String, (HashSet<usize>, HashSet<usize>)>,
    neighborhood: HashSet<String>,
}

impl EventHandler {
    pub fn new(id: usize, init_request: InitRequest) -> Self {
        let (node, node_ids) = match init_request {
            InitRequest::Init { node_id, node_ids } => (node_id, node_ids),
        };
        Self {
            node,
            id,
            known: node_ids
                .into_iter()
                .map(|nid| (nid, (HashSet::new(), HashSet::new())))
                .collect(),
            messages: HashSet::new(),
            neighborhood: HashSet::new(),
        }
    }
    pub fn handle_input_payload(&mut self, payload: BroadCastRequest) -> BroadCastRespone {
        match payload {
            BroadCastRequest::Broadcast { message } => {
                self.messages.insert(message);
                BroadCastRespone::BroadcastOk
            }
            BroadCastRequest::Read => BroadCastRespone::ReadOk {
                messages: self.messages.clone(),
            },
            BroadCastRequest::Topology { mut topology } => {
                if let Some(neighborhood) = topology.remove(&self.node) {
                    self.neighborhood = neighborhood.into_iter().collect();
                }
                BroadCastRespone::TopologyOk
            }
            BroadCastRequest::Gossip { .. } => todo!("this is required for multi-node"),
        }
    }
    pub fn handle_events<W: std::io::Write>(&mut self, rx: Receiver<Event>, writer: &mut W) {
        for event in rx.iter() {
            match event {
                Event::Tick => todo!("this is required for multi-node"),
                Event::Input(request) => {
                    let response = Message {
                        src: request.dst,
                        dst: request.src,
                        body: Body {
                            id: Some(self.id),
                            reply_id: request.body.id,
                            payload: self.handle_input_payload(request.body.payload),
                        },
                    };
                    response.send(writer);
                    self.id += 1;
                }
            };
        }
    }
}

#[allow(unreachable_code, unused_variables)]
pub fn ticker(event_tx: Sender<Event>, close_rx: Receiver<()>) {
    return;
    todo!("this is required for multi-node");
    let mut duration = 300;
    while let Err(RecvTimeoutError::Timeout) =
        close_rx.recv_timeout(Duration::from_millis(duration))
    {
        match event_tx.send(Event::Tick) {
            Ok(_) => duration = 300,
            Err(_) => duration = 10,
        }
    }
}

pub fn input_recv(event_tx: Sender<Event>, close_tx: SyncSender<()>) {
    let stdin = stdin().lock();
    let deseralizer = serde_json::Deserializer::from_reader(stdin);
    for input_request in deseralizer.into_iter().flatten() {
        if event_tx.send(Event::Input(input_request)).is_err() {
            break;
        }
    }
    drop(close_tx);
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
    let (close_tx, close_rx) = sync_channel(1);
    std::thread::spawn({
        let event_tx = event_tx.clone();
        move || ticker(event_tx, close_rx)
    });
    std::thread::spawn(move || input_recv(event_tx, close_tx));
    EventHandler::new(id, init_request).handle_events(event_rx, &mut stdout);
}
