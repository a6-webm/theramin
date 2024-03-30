use dioxus::prelude::*;
use tokio::sync::{mpsc, watch};

use crate::{
    input::InputHandler,
    manymouse::{self, Axis, Button, ManyMouse},
    midi::{MidiHandler, MidiInitialiser},
};

const MSG_BUFF_SIZE: usize = 30;
const DEFAULT_NOTE_WIDTH: u16 = 50;

type MsgTx = mpsc::Sender<Msg>;

pub enum Msg {
    FindNewDevices,
    ClickDev(usize),
}

pub struct TheraminMsgTx {
    tx: MsgTx,
}

impl TheraminMsgTx {
    pub fn send(&self, msg: Msg) {
        self.tx.try_send(msg).unwrap();
    }
}

type ThereminPositionsRx = watch::Receiver<ThereminPositions>;

type ThereminPositions = Vec<f32>;

pub type Devices = Vec<Dev>;

#[derive(Debug, Clone, PartialEq)]
pub struct Dev {
    pub id: usize,
    pub name: String,
    pub selected: bool,
    pub disconnected: bool,
}

struct DevHandlers {
    pos_idx: usize,
    input_h: InputHandler,
    midi_h: MidiHandler,
}

struct DevState {
    name: String,
    selected: Option<DevHandlers>,
    disconnected: bool,
}

fn gui_devices_from_states(dev_states: &[DevState]) -> Devices {
    dev_states
        .iter()
        .enumerate()
        .map(|(i, d_s)| Dev {
            id: i,
            name: d_s.name.clone(),
            selected: d_s.selected.is_some(),
            disconnected: d_s.disconnected,
        })
        .collect()
}

struct State {
    m_mouse: ManyMouse,
    dev_states: Vec<DevState>,
}

impl State {
    fn new(devs_tx: &watch::Sender<Devices>, pos_tx: &watch::Sender<ThereminPositions>) -> Self {
        let m_mouse = ManyMouse::new();
        let dev_states: Vec<DevState> = m_mouse
            .device_list()
            .into_iter()
            .map(|name| DevState {
                name,
                selected: None,
                disconnected: false,
            })
            .collect();
        devs_tx.send(gui_devices_from_states(&dev_states)).unwrap();
        pos_tx.send(Vec::new()).unwrap();
        State {
            m_mouse,
            dev_states,
        }
    }
}

pub fn use_theramin_msgs() -> Signal<TheraminMsgTx> {
    use_context()
}

pub fn use_theremin_positions() -> Signal<ThereminPositions> {
    let position_rx_context: Signal<ThereminPositionsRx> = use_context();
    use_update_context_by_rx(position_rx_context)
}

fn use_update_context_by_rx<T: Clone>(rx_sig: Signal<watch::Receiver<T>>) -> Signal<T> {
    let mut context: Signal<T> = use_context();
    use_future(move || async move {
        let mut rx = rx_sig.read().clone();
        loop {
            if let Err(_) = rx.changed().await {
                rx = rx_sig.read().clone();
            }
            *context.write() = rx.borrow_and_update().to_owned();
        }
    });
    context
}

pub fn use_theramin_routine() {
    // init with dummy channels
    let mut msg_tx_context = use_context_provider(|| {
        Signal::new(TheraminMsgTx {
            tx: mpsc::channel::<Msg>(1).0,
        })
    });
    let mut devices_rx_context =
        use_context_provider(|| Signal::new(watch::channel(Devices::new()).1));
    let mut positions_rx_context =
        use_context_provider(|| Signal::new(watch::channel(ThereminPositions::new()).1));

    use_context_provider(|| Signal::new(Devices::new()));
    use_context_provider(|| Signal::new(ThereminPositions::new()));

    use_hook(|| {
        // init with real channels
        let (msg_tx, mut msg_rx) = mpsc::channel(MSG_BUFF_SIZE);
        *msg_tx_context.write() = TheraminMsgTx { tx: msg_tx };
        let (devs_tx, devs_rx) = watch::channel(Devices::new());
        *devices_rx_context.write() = devs_rx;
        let (pos_tx, pos_rx) = watch::channel(ThereminPositions::new());
        *positions_rx_context.write() = pos_rx;

        tokio::spawn(async move {
            let mut s = State::new(&devs_tx, &pos_tx);
            'main_loop: loop {
                use mpsc::error::TryRecvError;
                loop {
                    match msg_rx.try_recv() {
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break 'main_loop,
                        Ok(msg) => {
                            match msg {
                                Msg::FindNewDevices => {
                                    s.dev_states
                                        .iter_mut()
                                        .filter_map(|d| d.selected.take().map(|hs| hs.midi_h))
                                        .for_each(|m_h| {
                                            m_h.close();
                                        });
                                    drop(s.m_mouse);
                                    s = State::new(&devs_tx, &pos_tx);
                                }
                                Msg::ClickDev(i) => {
                                    if s.dev_states[i].selected.is_some() {
                                        s.dev_states[i].selected.take().unwrap().midi_h.close();
                                    } else {
                                        s.dev_states[i].selected = Some(DevHandlers {
                                            pos_idx: 0,
                                            input_h: InputHandler::new(DEFAULT_NOTE_WIDTH),
                                            midi_h: MidiInitialiser::new()
                                                .virtual_port(&s.dev_states[i].name),
                                        });
                                    }
                                    // update pos_idxs
                                    s.dev_states
                                        .iter_mut()
                                        .filter_map(|d| d.selected.as_mut())
                                        .enumerate()
                                        .for_each(|(i, selected)| selected.pos_idx = i);
                                    // update device list
                                    devs_tx
                                        .send(gui_devices_from_states(&s.dev_states))
                                        .unwrap();
                                    // update length of pos array
                                    let new_len = s
                                        .dev_states
                                        .iter()
                                        .filter(|d| d.selected.is_some())
                                        .count();
                                    pos_tx.send_modify(|positions| positions.resize(new_len, 0.0));
                                }
                            }
                        }
                    }
                }
                for ev in s.m_mouse.poll() {
                    if s.dev_states[ev.device as usize].selected.is_none() {
                        continue;
                    }
                    match ev.ev_type {
                        manymouse::EventType::Relmotion if ev.item == Axis::X as u32 => {
                            let i = ev.device as usize;
                            let handlers = s.dev_states[i].selected.as_mut().unwrap();
                            let input_h = &mut handlers.input_h;
                            let midi_h = &mut handlers.midi_h;
                            let pitch = input_h.handle_rel_move(ev.value);
                            if input_h.playing {
                                midi_h.play(pitch);
                            }
                            pos_tx.send_modify(|positions| {
                                positions[handlers.pos_idx] = input_h.float_pos()
                            });
                        }
                        manymouse::EventType::Button if ev.item == Button::LMB as u32 => {
                            let i = ev.device as usize;
                            let handlers = s.dev_states[i].selected.as_mut().unwrap();
                            let input_h = &mut handlers.input_h;
                            let midi_h = &mut handlers.midi_h;
                            input_h.playing = ev.value == 1;
                            if input_h.playing {
                                midi_h.play(input_h.pitch_from_pos());
                            } else {
                                midi_h.release();
                            }
                        }
                        _ => (),
                    }
                }
            }
        });
    });
    use_update_context_by_rx(devices_rx_context);
}
