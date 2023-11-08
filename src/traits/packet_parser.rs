pub trait PacketParser<T>: Send + Sync {
    fn new() -> Self;
    fn clone_inner(&self) -> Self;
    fn parse_from_bytes(&mut self, data: &[u8]) -> Option<T>;
    fn parse_to_bytes(&mut self, data: &T) -> Vec<u8>;
}
