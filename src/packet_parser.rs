pub trait PacketParser<T>: Send + Sync {
    fn new() -> Self;
    fn clone(&self) -> Self;
    fn parse_from_bytes(&self, data: &[u8]) -> T;
    fn parse_to_bytes(&self, data: &T) -> Vec<u8>;
}
