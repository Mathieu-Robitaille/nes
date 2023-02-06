

#[cfg(test)]
mod ppu_tests{
    use super::{PPU, Cartridge};
    use crate::cartridge::Rom;
    use std::{rc::Rc, cell::RefCell};
    use tqdm::tqdm;
    
    #[test]
    fn test_ppu_write_register_0() {
        use crate::consts::ppu_consts::{
            CTRL_NAMETABLE_X, 
            CTRL_NAMETABLE_Y,
            REG_NAMETABLE_X,
            REG_NAMETABLE_Y
        };

        let cart: Rc<RefCell<Cartridge>> = Rc::new(RefCell::new(Cartridge::from(Rom::CPUTest).unwrap()));
        
        for data in tqdm(u8::MIN..=u8::MAX) {
            let mut ppu: PPU = PPU::new(cart.clone());
            ppu.cpu_write(0x2000, data);
    
            let data_nametable_x: u16 = ((data & CTRL_NAMETABLE_X) as u16) >> (CTRL_NAMETABLE_X.trailing_zeros());
            let data_nametable_y: u16 = ((data & CTRL_NAMETABLE_Y) as u16) >> (CTRL_NAMETABLE_Y.trailing_zeros());
    
            assert_eq!(data, ppu.ctrl.get_register());

            assert_eq!(data_nametable_x, ppu.tram_addr.nametable_x.get_as_value());
            assert_eq!(data_nametable_x, (ppu.tram_addr.nametable_x.get() & REG_NAMETABLE_X) >> REG_NAMETABLE_X.trailing_zeros());
            
            assert_eq!(data_nametable_y, ppu.tram_addr.nametable_y.get_as_value());
            assert_eq!(data_nametable_y, (ppu.tram_addr.nametable_y.get() & REG_NAMETABLE_Y) >> REG_NAMETABLE_Y.trailing_zeros());
        }
    }

    #[test]
    fn test_ppu_write_register_1() {
        let cart: Rc<RefCell<Cartridge>> = Rc::new(RefCell::new(Cartridge::from(Rom::CPUTest).unwrap()));
        
        for data in tqdm(u8::MIN..=u8::MAX) {
            let mut ppu: PPU = PPU::new(cart.clone());
            ppu.cpu_write(0x2001, data);
    
            assert_eq!(data, ppu.mask.get_register());
        }
    }

    #[test]
    fn test_ppu_write_register_6() {
        let cart: Rc<RefCell<Cartridge>> = Rc::new(RefCell::new(Cartridge::from(Rom::CPUTest).unwrap()));
        
        for data in tqdm(u16::MIN..=u16::MAX) {
            let mut ppu: PPU = PPU::new(cart.clone());

            ppu.cpu_write(0x2006, (data >> 8) as u8);
            ppu.cpu_write(0x2006, (data) as u8);
            assert_eq!(data, ppu.vram_addr.get_register());
        }
    }

    #[test]
    fn test_ppu_write_register_7() {
        let cart: Rc<RefCell<Cartridge>> = Rc::new(RefCell::new(Cartridge::from(Rom::CPUTest).unwrap()));
        
        let mut ppu: PPU = PPU::new(cart.clone());
        for i in (0..=1) {
            ppu.debug_set_ctrl_increment(i != 0);
            for data in tqdm(u8::MIN..=u8::MAX) {
                let vram_before_write = ppu.vram_addr.get_register();
    
                let (result, pass_fail) = ppu.vram_addr.get_register().overflowing_add(
                    if i == 1 { 32 } else { 1 }
                );

                ppu.cpu_write(0x2007, data);

                if !pass_fail {
                    assert_eq!(result, ppu.vram_addr.get_register());
                }

                // assert_eq!(data, ppu.ppu_read(vram_before_write));
            }
        }
    }

}