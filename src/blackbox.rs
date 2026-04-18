#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blackbox {
    pub x: i32,
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Blackbox {
    pub fn new() -> Self {
        Self { x: 0 }
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
        is_full::<Blackbox>();
    }
    #[test]
    fn new() {
        let blackbox = Blackbox::new();
        assert_eq!(0, blackbox.x);

    }
}
