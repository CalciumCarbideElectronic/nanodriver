use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::MutexGuard;

use crate::dac::ad537x::driver::AD5370Instance;
use crate::dac::ad537x::reg::ChannelAddress;
use crate::global::GLOBAL_AD5370;
#[derive(Debug)]
pub struct SinExeciter {
    freq: [f64; 40],
    amplitude: [u16; 40],
    done_ch: Receiver<Action>,
    iter: u128,
    sample_rate: u64,
}
#[allow(dead_code)]
pub enum Action {
    Stop,
    SetData { freq: f64, channel: u8, code: u16 },
}

impl SinExeciter {
    pub fn new(done_ch: Receiver<Action>) -> Self {
        Self {
            freq: [10.0; 40],
            amplitude: [0xF000; 40],
            done_ch,
            iter: 0,
            sample_rate: 175,
        }
    }
    #[inline]
    pub fn set_code_freq(&mut self, channel: u8, code: u16, freq: f64) {
        self.amplitude[channel as usize] = code;
        self.freq[channel as usize] = freq;
    }

    #[allow(dead_code)]
    fn inner_run_all(&mut self, lock: &mut MutexGuard<Box<dyn AD5370Instance>>) {
        self.iter += 1;
        let sample_per_period = 8000;
        let sample_index = (self.iter % sample_per_period as u128) as u64;
        let phase = sample_index as f64 / sample_per_period as f64;
        let amp = f64::sin((2_f64 * std::f64::consts::PI) * phase);
        let amp = amp + 1.0;
        let amp = amp * self.amplitude[0] as f64;
        let (amp, _) = u16::overflowing_add(amp.round() as u16, 0);
        lock.set_code(amp, ChannelAddress::AllCh).unwrap_or(());
        lock.clear_ldac().unwrap();
    }
    fn inner_run(&mut self, lock: &mut MutexGuard<Box<dyn AD5370Instance>>) {
        self.iter += 1;
        for (i, freq) in self.freq.iter().enumerate() {
            let sample_per_period = self.sample_rate as f64 / freq;
            let sample_index = self.iter as u64 % sample_per_period.round() as u64;
            let phase = sample_index as f64 / sample_per_period as f64;
            let amp = f64::sin((2_f64 * std::f64::consts::PI) * phase);
            let amp = (amp + 1.0) / 2.0;
            let amp = amp * self.amplitude[i] as f64;
            let (amp, _) = u16::overflowing_add(amp.round() as u16, 0);

            if lock
                .set_code(
                    amp,
                    ChannelAddress::SingleCh {
                        ch: i as u8 % 8,
                        group: i as u8 / 8,
                    },
                )
                .is_err()
            {
                std::process::exit(0);
            }
        }
    }
    pub fn run(&mut self) {
        let mut lock = GLOBAL_AD5370.lock().unwrap();
        lock.set_gain(0xF000).unwrap();
        lock.set_offset(0x8000).unwrap();
        lock.clear_ldac().unwrap();
        loop {
            self.inner_run(&mut lock);

            match self.done_ch.try_recv() {
                Ok(Action::Stop) | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }
                Err(TryRecvError::Empty) => {}
                Ok(Action::SetData {
                    freq,
                    channel,
                    code,
                }) => self.set_code_freq(channel, code, freq),
            }
        }
    }
}
