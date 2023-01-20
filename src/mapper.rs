pub trait MapperTrait {
    fn cpu_map_read(&self, addr: u16) -> Result<u32, ()>;
    fn ppu_map_read(&self, addr: u16) -> Result<u32, ()>;
    fn cpu_map_write(&self, addr: u16) -> Result<u32, ()>;
    fn ppu_map_write(&self, addr: u16) -> Result<u32, ()>;
}
pub struct Mapper000 {
    prg_banks: u8,
    chr_banks: u8,
}

impl Mapper000 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Self {
            prg_banks,
            chr_banks,
        }
    }
}

impl MapperTrait for Mapper000 {
    fn cpu_map_read(&self, addr: u16) -> Result<u32, ()> {
        if (0x8000..=0xFFFF).contains(&addr) {
            return Ok((addr & (if self.prg_banks > 1 { 0x7FFF } else { 0x3FFF })) as u32);
        }
        // println!("Errror {:?}", addr);
        Err(())
    }
    fn cpu_map_write(&self, addr: u16) -> Result<u32, ()> {
        if (0x8000..=0xFFFF).contains(&addr) {
            return Ok((addr & (if self.prg_banks > 1 { 0x7FFF } else { 0x3FFF })) as u32);
        }
        Err(())
    }
    fn ppu_map_read(&self, addr: u16) -> Result<u32, ()> {
        if (0x0000..=0x1FFF).contains(&addr) {
            return Ok(addr as u32);
        }
        Err(())
    }
    fn ppu_map_write(&self, addr: u16) -> Result<u32, ()> {
        // if (0x0000..=0x1FFF).contains(&addr) {
        //     Ok(0)
        // }
        Err(())
    }
}
