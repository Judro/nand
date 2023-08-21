use crate::bus::Bus;
use crate::connection::Connection;
use std::collections::HashMap;

pub trait Part<'s> {
    fn name(&self) -> String;
    fn load(&mut self, inputs: HashMap<&'s str, Connection<'s>>);
    fn simulate(&mut self) -> Vec<Bus>;
    fn outs(&self) -> Vec<Bus>;
}
