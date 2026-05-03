use crate::models::EmbPattern;

pub trait EmbroideryReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>>;
}
