use std::{
    io::{stdin, stdout},
    sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use gossip_glomers::{
    error::ErrorCode,
    init::{init, InitRequest},
    message::{Body, Message},
};
use serde::{Deserialize, Serialize};
// TODO: move these decoration to some macro.
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CounterRequest {
    Add {
        delta: usize,
    },
    Read,
    #[serde(rename = "read_ok")]
    KeyValue {
        value: usize,
    },
    #[serde(rename = "cas_ok")]
    UpdateSuccess,
    Error {
        code: ErrorCode,
        text: String,
    },
}

// TODO: move these decoration to some macro.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CounterRespone {
    AddOk,
    #[serde(rename = "read")]
    GetKey {
        key: String,
    },
    #[serde(rename = "cas")]
    UpdateValueFrom {
        key: String,
        #[serde(rename = "from")]
        old: usize,
        #[serde(rename = "to")]
        new: usize,
        #[serde(rename = "create_if_not_exists")]
        create: bool,
    },
    ReadOk {
        value: usize,
    },
}

const KV_NODE: &str = "seq-kv";
const KEY: &str = "COUNTER";

pub enum Event {
    Tick,
    Close,
    Input(Message<CounterRequest>),
}

#[allow(dead_code)]
struct EventHandler {
    id: usize,
    node: String,
    value: usize,
    delta: usize,
    last_update: Option<(usize, usize, usize)>,
}

impl EventHandler {
    pub fn new(id: usize, init_request: InitRequest) -> Self {
        let (node, _) = match init_request {
            InitRequest::Init { node_id, node_ids } => (node_id, node_ids),
        };

        Self {
            id,
            value: 0,
            delta: 0,
            node,
            last_update: None,
        }
    }

    fn handle_input_payload(
        &mut self,
        payload: CounterRequest,
        _src: &str,
        tick_tx: &mut Sender<()>,
    ) -> Option<CounterRespone> {
        match payload {
            CounterRequest::Add { delta } => {
                self.delta += delta;
                Some(CounterRespone::AddOk)
            }
            CounterRequest::Read => {
                tick_tx.send(()).expect("force ticking failed");
                Some(CounterRespone::ReadOk {
                    value: self.value + self.delta,
                })
            }
            CounterRequest::Error { code, .. } => match code {
                ErrorCode::KeyDoesNotExist => Some(CounterRespone::UpdateValueFrom {
                    key: KEY.into(),
                    old: 0,
                    new: 0,
                    create: true,
                }),
                ErrorCode::PreconditionFailed
                | ErrorCode::Timeout
                | ErrorCode::KeyAlreadyExists => {
                    if let Some((_, old, new)) = self.last_update.take() {
                        self.delta += new - old;
                        tick_tx.send(()).expect("force ticking failed");
                    };
                    None
                }
                error => panic!("Unhandled error code: {:?}", error),
            },
            CounterRequest::KeyValue { value } => {
                if self.delta > 0 {
                    self.value = value + self.delta;
                    self.last_update = Some((self.id, value, self.value));
                    let delta = self.delta;
                    self.delta = 0;
                    Some(CounterRespone::UpdateValueFrom {
                        key: KEY.into(),
                        old: value,
                        new: value + delta,
                        create: false,
                    })
                } else {
                    None
                }
            }
            CounterRequest::UpdateSuccess => {
                self.last_update.take();
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
                    let key = KEY.into();
                    let (payload, msg_id) = if let Some((msg_id, old, new)) = self.last_update {
                        (
                            CounterRespone::UpdateValueFrom {
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
                        (CounterRespone::GetKey { key }, id)
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

#[allow(unreachable_code, unused_variables)]
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

pub fn input_recv(event_tx: Sender<Event>) {
    {
        let stdin = stdin().lock();
        let deseralizer = serde_json::Deserializer::from_reader(stdin);
        for input_request in deseralizer.into_iter().flatten() {
            if event_tx.send(Event::Input(input_request)).is_err() {
                break;
            }
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
    EventHandler::new(id + 1, init_request).handle_events(event_rx, tick_tx, &mut stdout);
}
