#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct BlackboxConfig {
    pub fields_disabled_mask: u32,
    pub sample_rate: u8,
    pub device: BlackboxDevice,
    pub mode: BlackboxMode,
    pub high_resolution: u8,
    pub gps_use_3d_speed: bool,
    pub huffman_compress: bool,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BlackboxConfig {}

impl Default for BlackboxConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxConfig {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fields_disabled_mask: 0,
            sample_rate: 0,
            device: BlackboxDevice::NoDevice,
            mode: BlackboxMode::Normal,
            high_resolution: 0,
            gps_use_3d_speed: false,
            huffman_compress: false,
        }
    }
}
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
#[repr(u8)]
pub enum BlackboxDevice {
    #[default]
    NoDevice,
    Flash,
    SdCard,
    Serial,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BlackboxDevice {}

impl_try_from_u8!(BlackboxDevice);

#[allow(missing_docs)]
impl BlackboxDevice {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoDevice,
            1 => Self::Flash,
            2 => Self::SdCard,
            3 => Self::Serial,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
#[repr(u8)]
pub enum BlackboxMode {
    #[default]
    Normal,
    MotorTest,
    AlwaysOne,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BlackboxMode {}

impl_try_from_u8!(BlackboxMode);

#[allow(missing_docs)]
impl BlackboxMode {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::MotorTest,
            2 => Self::AlwaysOne,
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<BlackboxConfig>();
        is_full_eq::<BlackboxDevice>();
        is_full_eq::<BlackboxMode>();

        #[cfg(feature = "serde")]
        is_config::<BlackboxConfig>();
        #[cfg(feature = "serde")]
        is_config::<BlackboxDevice>();
        #[cfg(feature = "serde")]
        is_config::<BlackboxMode>();
    }
}
