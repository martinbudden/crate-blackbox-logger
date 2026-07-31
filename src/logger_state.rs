use crate::{
    BlackboxEvent::LoggingResume,
    BlackboxStartParameters, Logger, SliceEncoder,
    write_headers::{FieldHeaderIndex, SysInfoIndex},
};

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum LoggerState {
    #[default]
    Disabled = 0,
    Stopped,
    PrepareLogFile,
    WriteFileHeader,
    WriteMainFieldsHeader(FieldHeaderIndex),
    WriteGpsHFieldsHeader,
    WriteGpsGFieldsHeader,
    WriteSlowFieldsHeader,
    WriteSysinfo(SysInfoIndex),
    HeaderWritten,
    Paused,
    Running,
    ShuttingDown,
}

impl LoggerState {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self::Disabled
    }
}

impl LoggerState {
    /// Start logging.
    pub fn start(&mut self, _start_params: BlackboxStartParameters) {
        *self = Self::PrepareLogFile;
    }

    /// Finish logging.
    pub fn finish(&mut self) {
        *self = Self::ShuttingDown;
    }

    /// Allow state to be set directly. Used for debug and test.
    pub fn set_state(&mut self, state: Self) {
        *self = state;
    }

    /// Called each flight loop iteration to perform blackbox logging.
    pub fn update(&mut self, logger: &mut Logger, encoder: &mut SliceEncoder, current_time_us: u32) -> usize {
        *self = match core::mem::take(self) {
            // If we are disabled, we stay disabled until start() is called
            // Explicitly setting state = State::Disabled defends against a change in the default.
            Self::Disabled => Self::Disabled,
            Self::Stopped | Self::ShuttingDown => Self::Stopped,
            Self::PrepareLogFile => {
                logger.logged_any_frames = false;
                Self::WriteFileHeader
            }
            Self::WriteFileHeader => {
                Logger::write_file_header(encoder);
                Self::WriteMainFieldsHeader(FieldHeaderIndex::IName(0))
            }
            Self::WriteMainFieldsHeader(field_header) => {
                let next_field_header = logger.write_main_fields_header(encoder, field_header);
                if next_field_header == FieldHeaderIndex::End {
                    #[cfg(feature = "gps")]
                    if logger.enabled_fields & crate::field_definitions::FieldSelect::GPS != 0 {
                        Self::WriteGpsHFieldsHeader
                    } else {
                        Self::WriteSlowFieldsHeader
                    }
                    #[cfg(not(feature = "gps"))]
                    Self::WriteSlowFieldsHeader
                } else {
                    Self::WriteMainFieldsHeader(next_field_header)
                }
            }
            Self::WriteGpsHFieldsHeader => {
                #[cfg(feature = "gps")]
                logger.write_gps_g_fields_header(encoder);
                Self::WriteGpsGFieldsHeader
            }
            Self::WriteGpsGFieldsHeader => {
                #[cfg(feature = "gps")]
                logger.write_gps_h_fields_header(encoder);
                Self::WriteSlowFieldsHeader
            }
            Self::WriteSlowFieldsHeader => {
                logger.write_slow_fields_header(encoder);
                Self::WriteSysinfo(SysInfoIndex::Start)
            }
            Self::WriteSysinfo(sys_info) => {
                let next_sys_info = logger.write_sys_info(encoder, sys_info);
                if next_sys_info == SysInfoIndex::End { Self::HeaderWritten } else { Self::WriteSysinfo(next_sys_info) }
            }
            Self::HeaderWritten => Self::Paused,
            Self::Paused => {
                if logger.slow_data.is_blackbox_active() {
                    logger.force_log_i_frame();
                    logger.log_iteration(encoder, current_time_us);
                    logger.log_e_frame(encoder, LoggingResume(logger.iteration, current_time_us)); // must be after log_iteration
                    logger.advance_i_frame_indices();
                    Self::Running
                } else {
                    logger.advance_iteration_indices();
                    Self::Paused
                }
            }
            Self::Running => {
                if logger.slow_data.is_blackbox_active() {
                    logger.log_iteration(encoder, current_time_us);
                    logger.advance_iteration_indices();
                    Self::Running
                } else {
                    //logger.advance_iteration_timers();
                    Self::Paused
                }
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
    #[test]
    fn test_new() {
        let logger_state = LoggerState::new();
        assert_eq!(LoggerState::Disabled, logger_state);
    }
}
