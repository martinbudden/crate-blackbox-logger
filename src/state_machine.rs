use crate::{BlackboxStartParameters, Features, Logger, SliceWriter};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum StateMachine {
    #[default]
    Disabled = 0,
    Stopped,
    PrepareLogFile,
    LogFileHeader,
    LogMainFieldsHeader(usize),
    LogGpsHFieldsHeader,
    LogGpsGFieldsHeader,
    LogSlowFieldsHeader,
    LogSysinfo(usize),
    Paused,
    Running,
    ShuttingDown,
}

impl StateMachine {
    pub const fn new() -> Self {
        Self::Disabled
    }
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
                *self = StateMachine::LogFileHeader;
                0
            }
            StateMachine::LogFileHeader => {
                *self = StateMachine::LogMainFieldsHeader(0);
                Logger::log_file_header(writer)
            }
            StateMachine::LogMainFieldsHeader(index) => {
                let len = logger.log_main_fields_header(writer, index);
                if len == 0 {
                    *self = if logger.features.is_set(Features::GPS) {
                        StateMachine::LogGpsHFieldsHeader
                    } else {
                        StateMachine::LogSlowFieldsHeader
                    }
                } else {
                    *self = StateMachine::LogMainFieldsHeader(index + 1);
                }
                len
            }
            StateMachine::LogGpsHFieldsHeader => {
                *self = StateMachine::LogGpsGFieldsHeader;
                logger.log_gps_g_fields_header(writer)
            }
            StateMachine::LogGpsGFieldsHeader => {
                *self = StateMachine::LogSlowFieldsHeader;
                logger.log_gps_h_fields_header(writer)
            }
            StateMachine::LogSlowFieldsHeader => {
                *self = StateMachine::LogSysinfo(0);
                logger.log_slow_fields_header(writer)
            }
            StateMachine::LogSysinfo(index) => {
                let len = logger.log_sys_info(writer, index);
                *self = if len == 0 { StateMachine::Running } else { StateMachine::LogSysinfo(index + 1) };
                len
            }
            StateMachine::Paused => {
                *self = StateMachine::Paused;
                logger.advance_iteration_timers();
                0
            }
            StateMachine::Running => {
                *self = StateMachine::Running;
                let len = logger.log_iteration(current_time_us, writer);
                logger.advance_iteration_timers();
                len
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
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<StateMachine>();
    }
}
