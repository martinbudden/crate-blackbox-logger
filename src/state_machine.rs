use crate::{
    BlackboxStartParameters,
    Event::LoggingResume,
    Logger, SliceEncoder,
    log_headers::{FieldHeader, SysInfoIndex},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum StateMachine {
    #[default]
    Disabled = 0,
    Stopped,
    PrepareLogFile,
    LogFileHeader,
    LogMainFieldsHeader(FieldHeader),
    LogGpsHFieldsHeader,
    LogGpsGFieldsHeader,
    LogSlowFieldsHeader,
    LogSysinfo(SysInfoIndex),
    Paused,
    Running,
    ShuttingDown,
}

impl StateMachine {
    #[must_use]
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
    /// TODO: make this function asynchronous.
    pub fn update(
        &mut self,
        logger: &mut Logger,
        encoder: &mut SliceEncoder,
        current_time_us: u32,
        is_active: bool,
    ) -> usize {
        *self = match core::mem::take(self) {
            // If we are disabled, we stay disabled until start() is called
            // Explicitly setting state = State::Disabled defends against a change in the default.
            StateMachine::Disabled => StateMachine::Disabled,
            StateMachine::Stopped | StateMachine::ShuttingDown => StateMachine::Stopped,
            StateMachine::PrepareLogFile => {
                logger.logged_any_frames = false;
                StateMachine::LogFileHeader
            }
            StateMachine::LogFileHeader => {
                Logger::log_file_header(encoder);
                StateMachine::LogMainFieldsHeader(FieldHeader::IName)
            }
            StateMachine::LogMainFieldsHeader(field_header) => {
                let next_field_header = logger.log_main_fields_header(encoder, field_header);
                if next_field_header == FieldHeader::End {
                    if logger.features & Logger::FEATURE_GPS != 0 {
                        StateMachine::LogGpsHFieldsHeader
                    } else {
                        StateMachine::LogSlowFieldsHeader
                    }
                } else {
                    StateMachine::LogMainFieldsHeader(next_field_header)
                }
            }
            StateMachine::LogGpsHFieldsHeader => {
                #[cfg(feature = "gps")]
                {
                    logger.log_gps_g_fields_header(encoder);
                    StateMachine::LogGpsGFieldsHeader
                }
                #[cfg(not(feature = "gps"))]
                {
                    StateMachine::LogGpsGFieldsHeader
                }
            }
            StateMachine::LogGpsGFieldsHeader => {
                #[cfg(feature = "gps")]
                {
                    logger.log_gps_h_fields_header(encoder);
                    StateMachine::LogSlowFieldsHeader
                }
                #[cfg(not(feature = "gps"))]
                {
                    StateMachine::LogSlowFieldsHeader
                }
            }
            StateMachine::LogSlowFieldsHeader => {
                logger.log_slow_fields_header(encoder);
                StateMachine::LogSysinfo(SysInfoIndex::Start)
            }
            StateMachine::LogSysinfo(sys_info) => {
                let next_sys_info = logger.log_sys_info(encoder, sys_info);
                if next_sys_info == SysInfoIndex::End {
                    StateMachine::Running
                } else {
                    StateMachine::LogSysinfo(next_sys_info)
                }
            }
            StateMachine::Paused => {
                if is_active && logger.should_log_i_frame() {
                    logger.log_e_frame(encoder, LoggingResume(logger.iteration, current_time_us));
                    logger.log_iteration(current_time_us, encoder);
                    logger.advance_iteration_timers();
                    StateMachine::Running
                } else {
                    logger.advance_iteration_timers();
                    StateMachine::Paused
                }
            }
            StateMachine::Running => {
                logger.log_iteration(current_time_us, encoder);
                logger.advance_iteration_timers();
                StateMachine::Running
            }
        };
        encoder.pos
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
