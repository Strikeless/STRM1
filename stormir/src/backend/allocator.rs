use super::{target, Var};

pub struct Allocator {
    // This must be super inefficient and would not be viable for anything serious, probably a bottleneck to recode for future me.
    reg_bitmap: [bool; target::REG_COUNT],
    mem_bitmap: [bool; target::MEM_LENGTH],
}

// Very ugly, but does the job. I think.
impl Allocator {
    pub fn new() -> Self {
        Self {
            reg_bitmap: [false; target::REG_COUNT],
            mem_bitmap: [false; target::MEM_LENGTH],
        }
    }

    pub fn alloc(&mut self) -> Result<Var, ()> {
        self.alloc_reg()
            .map(|reg| Var::Register(reg))
            .or_else(|_| self.alloc_mem().map(|addr| Var::Memory(addr)))
    }

    pub fn free(&mut self, var: Var) {
        match var {
            Var::Register(reg) => self.free_reg(reg),
            Var::Memory(addr) => self.free_mem(addr),
        }
    }

    pub fn alloc_reg(&mut self) -> Result<usize, ()> {
        Self::bitmap_alloc(&mut self.reg_bitmap)
    }

    pub fn alloc_mem(&mut self) -> Result<usize, ()> {
        Self::bitmap_alloc(&mut self.mem_bitmap)
    }

    pub fn free_reg(&mut self, reg: usize) {
        Self::bitmap_free(&mut self.reg_bitmap, reg);
    }

    pub fn free_mem(&mut self, addr: usize) {
        Self::bitmap_free(&mut self.mem_bitmap, addr);
    }

    fn bitmap_alloc(map: &mut [bool]) -> Result<usize, ()> {
        let (index, used) = map
            .iter_mut()
            .enumerate()
            .find(|(_, used)| !**used)
            .ok_or(())?;

        *used = true;
        Ok(index)
    }

    fn bitmap_free(map: &mut [bool], index: usize) {
        assert!(map[index]);
        map[index] = false;
    }
}
