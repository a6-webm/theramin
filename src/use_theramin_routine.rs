use dioxus::prelude::*;
use tokio::sync::watch;

use crate::{
    input::InputHandler,
    manymouse::{self, Axis, Button, ManyMouse},
    midi::MidiInitialiser,
};

type MousePosTxs = Vec<watch::Sender<f32>>;

type MousePosRxs = Vec<watch::Receiver<f32>>;

pub type Devices = Vec<Dev>;

#[derive(Debug, Clone, PartialEq)]
pub struct Dev {
    pub id: usize,
    pub name: String,
    pub selected: bool,
}

impl Dev {
    fn new(id: usize, name: String) -> Self {
        Dev {
            id,
            name,
            selected: false,
        }
    }
}

pub fn use_mouse_pos(cx: &ScopeState, dev_id: usize) -> f32 {
    let rx_sh_state = use_shared_state::<MousePosRxs>(cx).unwrap();
    let dev_id_state = use_state(cx, || dev_id);
    let ref_rx = use_ref(cx, || rx_sh_state.read()[dev_id].clone());
    let mouse_pos = use_future(cx, ref_rx, |ref_rx| async move {
        ref_rx.write_silent().changed().await.unwrap();
        *ref_rx.write().borrow_and_update()
    });
    if *dev_id_state != dev_id {
        mouse_pos.cancel(cx);
        *ref_rx.write() = rx_sh_state.read()[dev_id].clone();
        dev_id_state.set(dev_id);
        mouse_pos.restart();
    }
    mouse_pos.value().cloned().unwrap_or(0.0)
}

pub fn use_theramin_routine(cx: &ScopeState) {
    use_shared_state_provider(cx, || MousePosRxs::new());
    use_shared_state_provider(cx, || Devices::new());
    let rx_sh_state = use_shared_state::<MousePosRxs>(cx).unwrap();
    let device_list = use_shared_state::<Devices>(cx).unwrap();
    cx.use_hook(|| {
        let mut m_mouse = ManyMouse::new();
        let mut m_pos_txs = MousePosTxs::new();
        for (i, s) in m_mouse.device_list().into_iter().enumerate() {
            device_list.write().push(Dev::new(i, s));
            let (tx, rx) = watch::channel(0.0);
            rx_sh_state.write().push(rx);
            m_pos_txs.push(tx);
        }
        tokio::spawn(async move {
            // TODO make midi ports and select devices by messages from the gui
            let mut midi_h = MidiInitialiser::new().virtual_port("port_1");
            let dbg_selected_devices = vec![0, 1, 2];
            let mut input_h = InputHandler::new(50);

            loop {
                for ev in m_mouse.poll() {
                    // println!("{:?}", ev);
                    if !dbg_selected_devices.contains(&ev.device) {
                        continue;
                    }
                    match ev.ev_type {
                        manymouse::EventType::Relmotion if ev.item == Button::LMB as u32 => {
                            let pitch = input_h.handle_rel_move(ev.value);
                            if input_h.playing {
                                midi_h.play(pitch);
                            }
                            m_pos_txs[ev.device as usize]
                                .send(input_h.float_pos())
                                .unwrap();
                        }
                        manymouse::EventType::Button if ev.item == Axis::X as u32 => {
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
}

// pub struct WatchTx<T> {
//     update: Arc<dyn Fn() + Send + Sync + 'static>,
//     tx: watch::Sender<T>,
// }

// impl<T> WatchTx<T> {
//     pub fn send(&self, value: T) -> Result<(), watch::error::SendError<T>> {
//         let res = self.tx.send(value);
//         if res.is_ok() {
//             (self.update)();
//         }
//         res
//     }
// }

// impl<T> UseWatchRx<T> {
//     pub fn read(&mut self) -> Result<watch::Ref<T>, watch::error::RecvError> {
//         match self.rx.has_changed() {
//             Ok(true) => Ok(self.rx.borrow_and_update()),
//             Ok(false) => Ok(self.rx.borrow()),
//             Err(e) => Err(e),
//         }
//     }
// }
