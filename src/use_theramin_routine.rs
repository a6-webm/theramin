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

impl Dev {
    fn new(id: usize, name: String) -> Self {
        Dev {
            id,
            name,
            selected: false,
            disconnected: false,
        }
    }
}

struct State {
    m_mouse: ManyMouse,
    devices: Devices,
    midi_hs: Vec<Option<MidiHandler>>,
    input_hs: Vec<Option<InputHandler>>,
}

impl State {
    fn new(devs_tx: &watch::Sender<Devices>, pos_tx: &watch::Sender<ThereminPositions>) -> Self {
        let m_mouse = ManyMouse::new();
        let devices: Devices = m_mouse
            .device_list()
            .into_iter()
            .enumerate()
            .map(|(i, s)| Dev::new(i, s))
            .collect();
        devs_tx.send(devices.clone()).unwrap();
        let mut midi_hs: Vec<Option<MidiHandler>> = Vec::new();
        midi_hs.resize_with(devices.len(), || None);
        let mut input_hs: Vec<Option<InputHandler>> = Vec::new();
        input_hs.resize_with(devices.len(), || None);
        // TODO make it so only the selected positions get sent
        pos_tx.send(vec![0.0; devices.len()]).unwrap();
        State {
            m_mouse,
            devices,
            midi_hs,
            input_hs,
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
                        Ok(msg) => match msg {
                            Msg::FindNewDevices => s = State::new(&devs_tx, &pos_tx),
                            Msg::ClickDev(idx) => {
                                if s.devices[idx].selected {
                                    s.input_hs[idx] = None;
                                    s.midi_hs[idx] = None;
                                } else {
                                    s.input_hs[idx] = Some(InputHandler::new(DEFAULT_NOTE_WIDTH));
                                    s.midi_hs[idx] = Some(
                                        MidiInitialiser::new().virtual_port(&s.devices[idx].name),
                                    );
                                }
                                s.devices[idx].selected = !s.devices[idx].selected;
                                devs_tx.send_modify(|devices| {
                                    devices[idx].selected = !devices[idx].selected
                                })
                            }
                        },
                    }
                }
                for ev in s.m_mouse.poll() {
                    if !s.devices[ev.device as usize].selected {
                        continue;
                    }
                    match ev.ev_type {
                        manymouse::EventType::Relmotion if ev.item == Axis::X as u32 => {
                            let dev_idx = ev.device as usize;
                            let input_h = &mut s.input_hs[dev_idx].as_mut().unwrap();
                            let midi_h = &mut s.midi_hs[dev_idx].as_mut().unwrap();
                            let pitch = input_h.handle_rel_move(ev.value);
                            if input_h.playing {
                                midi_h.play(pitch);
                            }
                            pos_tx
                                .send_modify(|positions| positions[dev_idx] = input_h.float_pos());
                        }
                        manymouse::EventType::Button if ev.item == Button::LMB as u32 => {
                            let dev_idx = ev.device as usize;
                            let input_h = &mut s.input_hs[dev_idx].as_mut().unwrap();
                            let midi_h = &mut s.midi_hs[dev_idx].as_mut().unwrap();
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
