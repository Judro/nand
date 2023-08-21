use crate::bus::Bus;

pub struct Connection<'s> {
    pub bus: Bus,
    pub chip_name: &'s str,
    pub source: &'s str,
    pub source_slice: (u8, u8),
    pub source_span: (u8, u8),
    pub dest: &'s str,
    pub dest_slice: (u8, u8),
    pub dest_span: (u8, u8),
}
