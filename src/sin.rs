use std::sync::mpsc::{Receiver, TryRecvError};

use crate::dac::ad537x::reg::ChannelAddress;
use crate::global::GLOBAL_AD5370;

pub struct SinExeciter {
    freq: [u64; 40],
    amplitude: [u16; 40],
    done_ch: Receiver<Action>,
    iter: u128,
    sample_rate: u64,
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
            iter: 0,
            sample_rate: 10_000,
        }
    }
    #[inline]
    pub fn set_freq(&mut self, channel: u8, freq: u64) {
        self.freq[channel as usize] = freq
    }
    #[inline]
    pub fn set_code(&mut self, channel: u8, code: u16) {
        self.amplitude[channel as usize] = code
    }

    fn inner_run(&mut self) {
        self.iter += 1;
        let mut lock = GLOBAL_AD5370.lock().unwrap();
        for (i, freq) in self.freq.iter().enumerate() {
            let sample_per_period = self.sample_rate / freq;
            let sample_index = (self.iter % sample_per_period as u128) as u64;
            let phase = (sample_index / sample_per_period) as f64;
            let amp = f64::sin((2_f64 * std::f64::consts::PI) * phase) * self.amplitude[i] as f64;
            lock.set_voltage(
                amp,
                ChannelAddress::SingleCh {
                    ch: i as u8 / 8,
                    group: i as u8 % 8,
                },
            )
            .unwrap_or(());
        }
    }
    pub fn run(&mut self) {
        loop {
            self.inner_run();

            match self.done_ch.try_recv() {
                Ok(Action::Stop) | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }
                Err(TryRecvError::Empty) => {}
                Ok(Action::SetFreq { freq, channel }) => self.set_freq(channel, freq),
                Ok(Action::SetAmp { code, channel }) => self.set_code(channel, code),
            }
        }
    }
}
