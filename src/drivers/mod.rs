pub mod sd_card;

#[allow(async_fn_in_trait)]
pub trait SdStorage {
    async fn write_all(&mut self, data: &[u8]) -> Result<(), ()>;
}
