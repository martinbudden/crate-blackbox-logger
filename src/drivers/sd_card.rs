use crate::drivers::SdStorage;
use std::fs::File;
use std::io::Write;
use embassy_futures::yield_now;


pub struct MockSdCard {
    file: File,
}

#[allow(clippy::expect_used)]
impl MockSdCard {
    /// # Panics
    pub fn new(path: &str) -> Self {
        Self {
            file: File::create(path).expect("Could not create log file"),
        }
    }
}

impl SdStorage for MockSdCard {
    async fn write_all(&mut self, data: &[u8]) -> Result<(), ()> {
        self.file.write_all(data).map_err(|_| ())?;
        _ = self.file.flush().ok();
        yield_now().await;
        Ok(())
    }
}