use bitflags::bitflags;

bitflags! {
    pub struct ALUFlags: u16 {
        const CARRY = 0b1;
        const ZERO  = 0b10;
    }
}
