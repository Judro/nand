use crate::bus::Bus;
use crate::connection::Connection;
use crate::part::Part;
use std::collections::HashMap;

pub struct Nand {
    a: Bus,
    b: Bus,
    out: Bus,
}

impl<'s> Part<'s> for Nand {
    fn name(&self) -> String {
        String::from("Nand")
    }

    fn load(&mut self, inputs: HashMap<&'s str, Connection>) {
        self.a = match inputs.get("a") {
            Some(b) => b.bus.pin(0),
            None => todo!(),
        };
        self.b = match inputs.get("b") {
            Some(b) => b.bus.pin(0),
            None => todo!(),
        };
    }

    fn simulate(&mut self) -> Vec<Bus> {
        self.out = Bus {
            data: !(self.a.data & self.b.data),
            size: 1,
        };
        vec![self.out]
    }

    fn outs(&self) -> Vec<Bus> {
        vec![self.out]
    }
}
