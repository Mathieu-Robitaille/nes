
use crate::consts::ppu_consts::*;

use num_traits::{FromPrimitive, PrimInt};
use std::ops::{AddAssign, SubAssign};
use std::marker::PhantomData;

#[derive(Debug, Default, Clone)]
pub struct Register<T>
where
    T: PrimInt + FromPrimitive,
{
    register: T,
    mask: T,
    size: usize,       // How many bits wide is this register
    left_shift: usize, // Where does this lie?
    phantom: PhantomData<T>,
}

impl<T> Register<T>
where
    T: PrimInt + FromPrimitive + AddAssign + SubAssign,
{
    fn new(data: T, mask: T) -> Self {
        let size: usize = mask.count_ones() as usize;
        let left_shift: usize = mask.trailing_zeros() as usize;

        let register: T = mask & data;
        Register {
            register,
            mask,
            size,
            left_shift,
            phantom: PhantomData
        }
    }
    pub fn get(&self) -> T {
        self.register
    }

    pub fn get_as_value(&self) -> T {
        self.register >> self.left_shift
    }

    pub fn set(&mut self, new: T) {
        self.register = new & self.mask;
    }

    pub fn set_with_unshifted(&mut self, new: T) {
        self.register = (new << self.left_shift) & self.mask;
    }

    pub fn flip_bits(&mut self) {
        self.register = (!self.register & self.mask);
    }

    pub fn zero(&mut self) {
        self.register = T::zero();
    }
    pub fn one(&mut self) {
        self.register = T::one() << self.left_shift;
    }

    pub fn increment(&mut self) {
        self.register += T::one();
    }
    pub fn decrement(&mut self) {
        self.register -= T::one();
    }
}

impl<T> Into<bool> for Register<T> 
where
    T: PrimInt + FromPrimitive,
{
    fn into(self) -> bool {
        self.register > T::zero()
    }
}

impl<T> std::cmp::PartialEq<T> for Register<T> 
where
    T: PrimInt + FromPrimitive + AddAssign + SubAssign,
{
    fn eq(&self, other: &T) -> bool {
        self.get_as_value() == *other
    }
}




///
/// Replacement register structs
/// I'm using these as a replacement for bitfields
/// 

#[derive(Debug, Default, Clone)]
pub struct StatusRegister {
    pub unused: Register<u8>,
    pub sprite_overflow: Register<u8>,
    pub sprite_zero_hit: Register<u8>,
    pub vertical_blank: Register<u8>,
}

impl From<u8> for StatusRegister {
    fn from(value: u8) -> Self {
        Self {
            unused: Register::new(value, 0b0001_1111),
            sprite_overflow: Register::new(value, 0b0010_0000),
            sprite_zero_hit: Register::new(value, 0b0100_0000),
            vertical_blank: Register::new(value, 0b1000_0000),
        }
    }
}

impl StatusRegister {
    pub fn get_register(&self) -> u8 {
        self.unused.get() 
        | self.sprite_overflow.get() 
        | self.sprite_zero_hit.get() 
        | self.vertical_blank.get() 
    }

    pub fn set_register(&mut self, new: u8) {
        self.unused.set(new);
        self.sprite_overflow.set(new);
        self.sprite_zero_hit.set(new);
        self.vertical_blank.set(new);
    }
}

#[derive(Debug, Default, Clone)]
pub struct ControlRegister {
    pub nametable_x: Register<u8>,
    pub nametable_y: Register<u8>,
    pub increment_mode: Register<u8>,
    pub pattern_sprite: Register<u8>,
    pub pattern_background: Register<u8>,
    pub sprite_size: Register<u8>,
    pub slave_mode: Register<u8>, // unused
    pub enable_nmi: Register<u8>,
}

impl From<u8> for ControlRegister {
    fn from(value: u8) -> Self {
        Self {
            nametable_x: Register::new(value, 0b0000_0001),
            nametable_y: Register::new(value, 0b0000_0010),
            increment_mode: Register::new(value, 0b0000_0100),
            pattern_sprite: Register::new(value, 0b0000_1000),
            pattern_background: Register::new(value, 0b0001_0000),
            sprite_size: Register::new(value, 0b0010_0000),
            slave_mode: Register::new(value, 0b0100_0000),
            enable_nmi: Register::new(value, 0b1000_0000),
        }
    }
}

impl ControlRegister {
    pub fn get_register(&self) -> u8 {
        self.nametable_x.get() 
        | self.nametable_y.get() 
        | self.increment_mode.get() 
        | self.pattern_sprite.get() 
        | self.pattern_background.get() 
        | self.sprite_size.get() 
        | self.slave_mode.get() 
        | self.enable_nmi.get() 
    }

    pub fn set_register(&mut self, new: u8) {
        self.nametable_x.set(new);
        self.nametable_y.set(new);
        self.increment_mode.set(new);
        self.pattern_sprite.set(new);
        self.pattern_background.set(new);
        self.sprite_size.set(new);
        self.slave_mode.set(new);
        self.enable_nmi.set(new);
    }
}

#[derive(Debug, Default, Clone)]
pub struct MaskRegister {
    pub grayscale: Register<u8>,
    pub render_background_left: Register<u8>,
    pub render_sprites_left: Register<u8>,
    pub render_background: Register<u8>,
    pub render_sprites: Register<u8>,
    pub enhance_red: Register<u8>,
    pub enhance_green: Register<u8>,
    pub enhance_blue: Register<u8>,
}

impl From<u8> for MaskRegister {
    fn from(value: u8) -> Self {
        Self {
            grayscale: Register::new(value, 0b0000_0001),
            render_background_left: Register::new(value, 0b0000_0010),
            render_sprites_left: Register::new(value, 0b0000_0100),
            render_background: Register::new(value, 0b0000_1000),
            render_sprites: Register::new(value, 0b0001_0000),
            enhance_red: Register::new(value, 0b0010_0000),
            enhance_green: Register::new(value, 0b0100_0000),
            enhance_blue: Register::new(value, 0b1000_0000),
        }
    }
}

impl MaskRegister {
    pub fn get_register(&self) -> u8 {
        self.grayscale.get() 
        | self.render_background_left.get() 
        | self.render_sprites_left.get() 
        | self.render_background.get() 
        | self.render_sprites.get() 
        | self.enhance_red.get() 
        | self.enhance_green.get() 
        | self.enhance_blue.get() 
    }

    pub fn set_register(&mut self, new: u8) {
        self.grayscale.set(new);
        self.render_background_left.set(new);
        self.render_sprites_left.set(new);
        self.render_background.set(new);
        self.render_sprites.set(new);
        self.enhance_red.set(new);
        self.enhance_green.set(new);
        self.enhance_blue.set(new);
    }
}

#[derive(Debug, Default, Clone)]
pub struct VramRegister {
    pub coarse_x: Register<u16>,
    pub coarse_y: Register<u16>,
    pub nametable_x: Register<u16>,
    pub nametable_y: Register<u16>,
    pub fine_y: Register<u16>,
    pub unused: Register<u16>,
}

impl From<u16> for VramRegister {
    fn from(value: u16) -> Self {
        Self {
            coarse_x: Register::new(value, REG_COARSE_X),
            coarse_y: Register::new(value, REG_COARSE_Y),
            nametable_x: Register::new(value, REG_NAMETABLE_X),
            nametable_y: Register::new(value, REG_NAMETABLE_Y),
            fine_y: Register::new(value, REG_FINE_Y),
            unused: Register::new(value, REG_UNUSED),
        }
    }
}

impl VramRegister {
    pub fn get_register(&self) -> u16 {
        self.coarse_x.get() 
        | self.coarse_y.get() 
        | self.nametable_x.get() 
        | self.nametable_y.get() 
        | self.fine_y.get() 
        | self.unused.get() 
    }

    pub fn set_register(&mut self, new: u16) {
        self.coarse_x.set(new);
        self.coarse_y.set(new);
        self.nametable_x.set(new);
        self.nametable_y.set(new);
        self.fine_y.set(new);
        self.unused.set(new);
    }
}

