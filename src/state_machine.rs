use crate::{
    BlackboxStartParameters,
    Event::LoggingResume,
    Logger, SliceEncoder,
    log_headers::{FieldHeaderIndex, SysInfoIndex},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum LoggerState {
    #[default]
    Disabled = 0,
    Stopped,
    PrepareLogFile,
    LogFileHeader,
    LogMainFieldsHeader(FieldHeaderIndex),
    LogGpsHFieldsHeader,
    LogGpsGFieldsHeader,
    LogSlowFieldsHeader,
    LogSysinfo(SysInfoIndex),
    Paused,
    Running,
    ShuttingDown,
}

impl LoggerState {
    #[must_use]
    pub const fn new() -> Self {
        Self::Disabled
    }
}

impl LoggerState {
    pub fn start(&mut self, _start_params: BlackboxStartParameters) {
        *self = LoggerState::PrepareLogFile;
    }

    pub fn finish(&mut self) {
        *self = LoggerState::ShuttingDown;
    }

    pub fn set_state(&mut self, state: Self) {
        *self = state;
    }

    /// Called each flight loop iteration to perform blackbox logging.
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
            LoggerState::Disabled => LoggerState::Disabled,
            LoggerState::Stopped | LoggerState::ShuttingDown => LoggerState::Stopped,
            LoggerState::PrepareLogFile => {
                logger.logged_any_frames = false;
                LoggerState::LogFileHeader
            }
            LoggerState::LogFileHeader => {
                Logger::log_file_header(encoder);
                LoggerState::LogMainFieldsHeader(FieldHeaderIndex::IName(0))
            }
            LoggerState::LogMainFieldsHeader(field_header) => {
                let next_field_header = logger.log_main_fields_header(encoder, field_header);
                if next_field_header == FieldHeaderIndex::End {
                    if logger.features & Logger::FEATURE_GPS != 0 {
                        LoggerState::LogGpsHFieldsHeader
                    } else {
                        LoggerState::LogSlowFieldsHeader
                    }
                } else {
                    LoggerState::LogMainFieldsHeader(next_field_header)
                }
            }
            LoggerState::LogGpsHFieldsHeader => {
                #[cfg(feature = "gps")]
                {
                    logger.log_gps_g_fields_header(encoder);
                }
                LoggerState::LogGpsGFieldsHeader
            }
            LoggerState::LogGpsGFieldsHeader => {
                #[cfg(feature = "gps")]
                {
                    logger.log_gps_h_fields_header(encoder);
                }
                LoggerState::LogSlowFieldsHeader
            }
            LoggerState::LogSlowFieldsHeader => {
                logger.log_slow_fields_header(encoder);
                LoggerState::LogSysinfo(SysInfoIndex::Start)
            }
            LoggerState::LogSysinfo(sys_info) => {
                let next_sys_info = logger.log_sys_info(encoder, sys_info);
                if next_sys_info == SysInfoIndex::End {
                    LoggerState::Running
                } else {
                    LoggerState::LogSysinfo(next_sys_info)
                }
            }
            LoggerState::Paused => {
                if is_active && logger.should_log_i_frame() {
                    logger.log_e_frame(encoder, LoggingResume(logger.iteration, current_time_us));
                    logger.log_iteration(encoder, current_time_us);
                    logger.advance_iteration_timers();
                    LoggerState::Running
                } else {
                    logger.advance_iteration_timers();
                    LoggerState::Paused
                }
            }
            LoggerState::Running => {
                logger.log_iteration(encoder, current_time_us);
                logger.advance_iteration_timers();
                LoggerState::Running
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
        is_full::<LoggerState>();
    }
}
