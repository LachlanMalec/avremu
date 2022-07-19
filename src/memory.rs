use std::cell::{RefCell, RefMut};
use std::cmp::Ordering;
use std::rc::Rc;

pub trait MemoryMapped {
    fn get_size(&self) -> usize;
    fn read(&self, address: usize) -> (u8, usize);
    fn write(&mut self, address: usize, value: u8) -> usize;
    fn read_word(&self, address: usize) -> (u16, usize) {
        let (bl, ll) =  self.read(address);
        let (bh, lh) =  self.read(address+1);
        (((bh as u16) << 8) | (bl as u16), ll+lh)
    }
    fn write_word(&mut self, address: usize, value: u16) -> usize {
        let bl =  value as u8;
        let bh =  (value >> 8) as u8;
        self.write(address, bl) + self.write(address+1, bh)
    }
    fn set_bit(&mut self, address: usize, bit: u8, state: bool) -> usize {
        let (val, _) = self.read(address);
        if state {
            self.write(address, val | (1 << bit));
        } else {
            self.write(address, val & !(1 << bit));
        }
        0
    }
}

pub struct MemoryMap {
    mm: Vec<(usize, Rc<RefCell<dyn MemoryMapped>>)>
}

impl MemoryMap {
    pub fn new() -> Self {
        MemoryMap { mm: Vec::new() }
    }

    pub fn add(&mut self, offset: usize, dev: Rc<RefCell<dyn MemoryMapped>>) {
        self.mm.push((offset, dev));
    }

    fn get_dev(&self, address: usize) -> (RefMut<dyn MemoryMapped>, usize)  {
        //println!("get_dev: Got request to find device at 0x{:04X}", address);
        let idx = self.mm.binary_search_by(|dev| {
            if address < dev.0 {
                Ordering::Greater
            } else if address >= dev.0+dev.1.borrow().get_size() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }).expect("Attempt to access undefined region of memory map.");
        //println!("get_dev: Found a device at 0x{:04X}, idx = {}", self.mm[idx].0, idx);
        (self.mm[idx].1.borrow_mut(), address-self.mm[idx].0)
    }
}

impl MemoryMapped for MemoryMap {
    fn get_size(&self) -> usize {
        self.mm.len()
    }

    fn read(&self, address: usize) -> (u8, usize) {
        let (dev, offset) = self.get_dev(address);
        dev.read(offset)
    }

    fn write(&mut self, address: usize, value: u8) -> usize {
        let (mut dev, offset) = self.get_dev(address);
        dev.write(offset, value)
    }
}



pub struct Memory {
    mem: Vec<u8>,
    lat: usize,
    read_only: bool
}

impl Memory {
    pub fn new(size: usize, fill: u8, lat: usize) -> Self {
        Memory {
            mem: vec![fill; size],
            lat, 
            read_only: false
        }
    }

    pub fn new_rom(mem: Vec<u8>, lat: usize) -> Self {
        Memory {
            mem,
            lat, 
            read_only: true
        }
    }
}

impl MemoryMapped for Memory {
    fn get_size(&self) -> usize {
        self.mem.len()
    }

    fn read(&self, address: usize) -> (u8, usize) {
        (self.mem[address], self.lat)
    }

    fn write(&mut self, address: usize, value: u8) -> usize {
        if !self.read_only {
            self.mem[address] = value;
        }
        0
    }
}