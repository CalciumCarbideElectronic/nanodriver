use std::sync::mpsc::{Receiver, TryRecvError};

pub struct SinExeciter {
    freq: [u64; 40],
    amplitude: [u16; 40],
    done_ch: Receiver<Action>,
}
pub enum Action {
    Stop,
    SetFreq { freq: u64, channel: u8 },
    SetAmp { code: u16, channel: u8 },
}

impl SinExeciter {
    pub fn new(done_ch: Receiver<Action>) -> Self {
        Self {
            freq: [1000_u64; 40],
            amplitude: [0x8888; 40],
            done_ch,
        }
    }
    pub fn run(&self) {
        loop {
            match self.done_ch.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}
