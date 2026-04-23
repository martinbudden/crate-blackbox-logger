use crate::{BLACKBOX_MAIN_FIELDS, BlackboxStart, BlackboxWriter, SliceWriter, write_main_header};
use vqm::BitSet64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxContext {
    pub(crate) logged_any_frames: bool,
    pub(crate) conditions: BitSet64,
    pub(crate) i_interval: u32,
    pub(crate) p_interval: u32,
    looptime: u32,
}

impl Default for BlackboxContext {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxContext {
    pub fn new() -> Self {
        Self { logged_any_frames: false, conditions: BitSet64::default(), i_interval: 0, p_interval: 0, looptime: 125 }
    }
}

impl BlackboxContext {
    pub fn send_header(writer: &mut SliceWriter) -> usize {
        writer.write_h_str("Product:Blackbox flight data recorder by Nicholas Sherlock\n");
        writer.write_h_str("Data version:2\n");
        writer.pos
    }

    pub fn send_main_field_header(&mut self, writer: &mut SliceWriter) -> usize {
        write_main_header(writer, BLACKBOX_MAIN_FIELDS, self.conditions);
        writer.pos
    }

    pub fn send_sys_header(&mut self, writer: &mut SliceWriter, index: usize) -> usize {
        match index {
            0 => {
                writer.write_h_str("Firmware type:Cleanflight\n");
                writer.pos
            }
            1 => {
                writer.write_h_str("Firmware revision:Betaflight 3.3.1 (611bc70f8) REVOLT\n");
                writer.pos
            }
            2 => {
                writer.write_h_str("Firmware date:Mar 21 2018 00:00:00\n");

                writer.pos
            }
            3 => {
                writer.write_h_str("Log start datetime:0000-01-01T00:00:00.000+00:00\n");
                writer.pos
            }
            4 => {
                writer.write_h_str("Craft name:Protea\n");
                writer.pos
            }
            5 => {
                writer.write_h_str("I interval:");
                writer.write_u32_ascii(self.i_interval);
                writer.write_char('\n');
                writer.pos
            }
            6 => {
                writer.write_h_str("P interval:");
                writer.write_u32_ascii(self.p_interval);
                writer.write_char('\n');
                writer.pos
                // "P denom" ignored by blackbox-log-view
                // writer.write_h_str("P denom:32\n");
            }
            7 => {
                writer.write_h_str("looptime:");
                writer.write_u32_ascii(self.looptime);
                writer.write_char('\n');
                writer.pos
            }
            8 => {
                writer.write_h_str("gyro_sync_denom:1\n");
                writer.write_h_str("pid_process_denom:1\n");

                writer.write_h_str("gyro_scale:0x3f800000\n");
                writer.write_h_str("acc_1G:4096\n");

                writer.write_h_str("features:541130760\n");
                writer.write_h_str("debug_mode:0\n");

                writer.write_h_str("minthrottle:1070\n");
                writer.write_h_str("maxthrottle:2000\n");
                writer.write_h_str("motorOutput:158,2047\n");

                writer.write_h_str("vbat_scale:110\n");
                writer.write_h_str("vbatcellvoltage:33,35,43\n");
                writer.write_h_str("vbatref:113\n");
                writer.write_h_str("currentSensor:0,235\n");
                writer.pos
            }
            _ => 0,
        }
        /*

        writer.pos*/
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub(crate) enum State {
    #[default]
    Disabled = 0,
    Stopped,
    PrepareLogFile,
    SendHeader,
    SendMainFieldHeader,
    SendGpsHHeader,
    SendGpsGHeader,
    SendSlowHeader,
    SendSysinfo(usize),
    Paused,
    Running,
    ShuttingDown,
}

#[allow(dead_code)]
// Note: Not sure if this state machine is needed: it might naturally drop out of the embassy sync framework.
impl State {
    pub fn start(&mut self, _start_params: BlackboxStart) {
        *self = State::SendSysinfo(0);
    }

    pub fn finish(&mut self) {
        *self = State::ShuttingDown;
    }

    pub fn set_state(&mut self, state: Self) {
        *self = state;
    }

    /// Called each flight loop iteration to perform blackbox logging.
    pub fn update(&mut self, ctx: &mut BlackboxContext, writer: &mut SliceWriter) -> usize {
        #[allow(clippy::match_same_arms)]
        match core::mem::take(self) {
            State::Disabled => {
                // If we are disabled, we stay disabled until start() is called
                // Explicitly setting *self = State::Disabled defends against a change in the default.
                *self = State::Disabled;
                0
            }
            State::Stopped => {
                *self = State::Stopped;
                0
            }
            State::PrepareLogFile => {
                ctx.logged_any_frames = false;
                *self = State::SendHeader;
                0
            }
            State::SendHeader => {
                *self = State::SendMainFieldHeader;
                BlackboxContext::send_header(writer)
            }
            State::SendMainFieldHeader => {
                let len = ctx.send_main_field_header(writer);
                if len == 0 {
                    *self = State::SendSlowHeader;
                    //*self = State::SendGpsHHeader;
                }
                len
            }
            State::SendGpsHHeader => {
                *self = State::SendGpsGHeader;
                0
            }
            State::SendGpsGHeader => {
                *self = State::SendSlowHeader;
                0
            }
            State::SendSlowHeader => {
                *self = State::SendSysinfo(0);
                0
            }
            State::SendSysinfo(index) => {
                let len = ctx.send_sys_header(writer, index);
                if len == 0 {
                    *self = State::Running;
                    0
                } else {
                    *self = State::SendSysinfo(index + 1);
                    len
                }
            }
            State::Paused => {
                *self = State::Running;
                0
            }
            State::Running => {
                *self = State::Paused;
                0
            }
            State::ShuttingDown => {
                *self = State::Stopped;
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    #![allow(unused_results)]

    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<BlackboxContext>();
    }
    #[test]
    fn new() {
        let ctx = BlackboxContext::new();
        assert!(!ctx.logged_any_frames);
    }
    #[test]
    fn test_send_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };

        _ = BlackboxContext::send_header(&mut writer);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nHEADER\r\n{result}\n");
        assert!(result.contains("H Product:Blackbox"));
    }
    #[test]
    fn send_main_field_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = BlackboxContext::new();

        _ = ctx.send_main_field_header(&mut writer);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nMAIN FIELD HEADER\r\n{result}\n");
    }
    #[test]
    fn send_sys_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = BlackboxContext::new();

        let mut index: usize = 0;
        loop {
            if ctx.send_sys_header(&mut writer, index) == 0 {
                break;
            }
            index += 1;
        }

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSYS HEADER\r\n{result}\n");
    }
    #[test]
    fn state_machine() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = BlackboxContext::new();

        let start = BlackboxStart::new();
        let mut state = State::default();
        state.start(start);
        loop {
            _ = state.update(&mut ctx, &mut writer);
            //let state_i:u32 = state.into();
            //println!("state={state_i}");
            if state == State::Running {
                if writer.pos != 0 {
                    #[allow(clippy::unwrap_used)]
                    let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
                    println!("{result}");
                }
                break;
            }
        }
    }
}
