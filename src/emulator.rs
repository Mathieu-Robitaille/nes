pub trait EmulatedDevice {
    fn reset(&mut self);
    fn clock(&mut self);
}

pub trait BusDevice {
    fn read_one_byte(&mut self, addr: u16, ro: bool) -> u8;
    fn write_byte(&mut self, addr: u16, data: u8, ro: bool);
}

pub trait DisplayDevice {
    fn display_read_one_byte(&mut self, addr: u16, ro: bool) -> u8;
    fn display_write_byte(&mut self, addr: u16, data: u8, ro: bool);
}
