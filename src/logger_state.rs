use crate::{
    BlackboxEvent::LoggingResume,
    Logger, SliceEncoder,
    write_headers::{FieldHeaderIndex, SysInfoIndex},
};

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum LoggerState {
    #[default]
    Disabled = 0,
    Stopped,
    Start(u16),
    WriteFileHeader,
    WriteMainFieldsHeader(FieldHeaderIndex),
    WriteGpsHFieldsHeader,
    WriteGpsGFieldsHeader,
    WriteSlowFieldsHeader,
    WriteSysinfo(SysInfoIndex),
    WriteHuffmanTable,
    HeaderWritten,
    Paused,
    Running,
    ShuttingDown,
    // CacheFlush,
    // StartErase,
    // Erasing,
    // Erased,
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
    pub fn start(&mut self, debug_mode: u16) {
        *self = Self::Start(debug_mode);
    }

    /// Finish logging.
    pub fn finish(&mut self) {
        *self = Self::ShuttingDown;
    }

    /// Allow state to be set directly. Used for debug and test.
    pub fn set_state(&mut self, state: Self) {
        *self = state;
    }

    /// Helper function used when writing header.
    pub fn update_header(&mut self, logger: &mut Logger, encoder: &mut SliceEncoder, current_time_us: u64) -> usize {
        self.update(logger, encoder, current_time_us, false)
    }

    /// Called each flight loop iteration to perform blackbox logging.
    pub fn update(
        &mut self,
        logger: &mut Logger,
        encoder: &mut SliceEncoder,
        current_time_us: u64,
        force_i_frame: bool,
    ) -> usize {
        *self = match core::mem::take(self) {
            // If we are disabled, we stay disabled until start() is called
            // Explicitly setting state = State::Disabled defends against a change in the default.
            Self::Disabled => Self::Disabled,
            Self::Stopped | Self::ShuttingDown => Self::Stopped,
            Self::Start(debug_mode) => {
                logger.logged_any_frames = false;
                logger.debug_mode = debug_mode;
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
            Self::WriteSysinfo(sys_info_index) => {
                let sys_info_index = logger.write_sys_info(encoder, sys_info_index);
                if sys_info_index == SysInfoIndex::End {
                    Self::WriteHuffmanTable
                } else {
                    Self::WriteSysinfo(sys_info_index)
                }
            }
            Self::WriteHuffmanTable => {
                #[cfg(feature = "huffman")]
                if logger.huffman_compress {
                    logger.log_t_frame(encoder);
                }
                Self::HeaderWritten
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
                    if force_i_frame {
                        logger.force_log_i_frame();
                    }
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
