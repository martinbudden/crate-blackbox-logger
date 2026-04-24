use crate::{BlackboxStartParameters, Features, Logger, SliceWriter};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum StateMachine {
    #[default]
    Disabled = 0,
    Stopped,
    PrepareLogFile,
    SendHeader,
    SendMainFieldHeader(usize),
    SendGpsHHeader,
    SendGpsGHeader,
    SendSlowHeader,
    SendSysinfo(usize),
    Paused,
    Running,
    ShuttingDown,
}

impl StateMachine {
    pub fn start(&mut self, _start_params: BlackboxStartParameters) {
        *self = StateMachine::PrepareLogFile;
    }

    pub fn finish(&mut self) {
        *self = StateMachine::ShuttingDown;
    }

    pub fn set_state(&mut self, state: Self) {
        *self = state;
    }

    /// Called each flight loop iteration to perform blackbox logging.
    pub fn update(&mut self, logger: &mut Logger, writer: &mut SliceWriter, current_time_us: u32) -> usize {
        #[allow(clippy::match_same_arms)]
        match core::mem::take(self) {
            StateMachine::Disabled => {
                // If we are disabled, we stay disabled until start() is called
                // Explicitly setting *self = State::Disabled defends against a change in the default.
                *self = StateMachine::Disabled;
                0
            }
            StateMachine::Stopped => {
                *self = StateMachine::Stopped;
                0
            }
            StateMachine::PrepareLogFile => {
                logger.logged_any_frames = false;
                *self = StateMachine::SendHeader;
                0
            }
            StateMachine::SendHeader => {
                *self = StateMachine::SendMainFieldHeader(0);
                Logger::log_header(writer)
            }
            StateMachine::SendMainFieldHeader(index) => {
                let len = logger.log_main_field_header(writer, index);
                if len == 0 {
                    *self = if logger.features.is_set(Features::GPS) {
                        StateMachine::SendGpsHHeader
                    } else {
                        StateMachine::SendSlowHeader
                    }
                } else {
                    *self = StateMachine::SendMainFieldHeader(index + 1);
                }
                len
            }
            StateMachine::SendGpsHHeader => {
                *self = StateMachine::SendGpsGHeader;
                //logger.log_gps_g_header(writer)
                0
            }
            StateMachine::SendGpsGHeader => {
                *self = StateMachine::SendSlowHeader;
                //logger.log_gps_h_header(writer)
                0
            }
            StateMachine::SendSlowHeader => {
                *self = StateMachine::SendSysinfo(0);
                logger.log_slow_header(writer)
            }
            StateMachine::SendSysinfo(index) => {
                let len = logger.log_sys_header(writer, index);
                *self = if len == 0 { StateMachine::Running } else { StateMachine::SendSysinfo(index + 1) };
                len
            }
            StateMachine::Paused => {
                *self = StateMachine::Running;
                0
            }
            StateMachine::Running => {
                //*self = State::Paused;
                logger.log_iteration(current_time_us, writer);
                0
            }
            StateMachine::ShuttingDown => {
                *self = StateMachine::Stopped;
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<StateMachine>();
    }
}
