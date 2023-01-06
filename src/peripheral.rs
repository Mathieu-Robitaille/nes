trait Peripheral {
    fn low_end(&self) -> u16;
    fn high_end(&self) -> u16;
    fn entire_range(&self) -> [u16];
}

