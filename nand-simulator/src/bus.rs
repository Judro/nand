use thiserror::Error;

/// can contain 1 to 128 pins: A[0] .. A[0..129]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Bus {
    pub data: u128,
    pub size: u8,
}

#[derive(Error, Debug, PartialEq)]
pub enum BusError {
    #[error("A bus should not contain more than 128 pins!")]
    SourceBusMaxExceeded,
    #[error("A bus should not contain more than 128 pins!")]
    DestinationBusMaxExceeded,
}

fn mask(slice: (u8, u8)) -> u128 {
    // TODO fails when > 128,129 or (0,0)
    u128::MAX >> (128 - (slice.1 - slice.0)) << slice.0
}

impl Bus {
    pub fn load(
        &mut self,
        source_bus: &Bus,
        source: (u8, u8),
        dest: (u8, u8),
    ) -> Result<(), BusError> {
        if source.0 > 128 || source.1 > 129 {
            return Err(BusError::SourceBusMaxExceeded);
        }
        if dest.0 > 128 || dest.1 > 129 {
            return Err(BusError::DestinationBusMaxExceeded);
        }
        let mut tmp = source_bus.data;
        let shift: u8;
        if dest.0 > source.0 {
            shift = dest.0 - source.0;
            tmp = tmp << shift & mask(dest);
        } else {
            shift = source.0 - dest.0;
            tmp = tmp >> shift & mask(dest);
        }
        self.data = self.data ^ (self.data & mask(dest)) | tmp;
        Ok(())
    }

    pub fn pin(&self, pin: u8) -> Bus {
        Bus {
            data: self.data << (127 - pin) >> 127,
            size: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_0() {
        assert_eq!(0b1100, mask((2, 4)));
        assert_eq!(0b0011, mask((0, 2)));
        assert_eq!(0b0001, mask((0, 1)));
        // A Bus can only contain 128 Pins
        assert_eq!(u128::MAX, mask((0, 128)));
    }

    #[test]
    fn load_0() {
        let mut a = Bus {
            data: 0b00000000,
            size: 8,
        };
        let b = Bus {
            data: 0b11011100,
            size: 8,
        };
        let res = a.load(&b, (2, 7), (0, 5));
        assert_eq!(a.data, 0b10111);
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn load_1() {
        let mut a = Bus {
            data: 0b00000011,
            size: 8,
        };
        let b = Bus {
            data: 0b11011100,
            size: 8,
        };
        let res = a.load(&b, (0, 5), (2, 7));
        assert_eq!(a.data, 0b01110011);
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn load_2() {
        let mut a = Bus {
            data: 0b00000011,
            size: 8,
        };
        let b = Bus {
            data: 0b11011101,
            size: 8,
        };
        let res = a.load(&b, (0, 5), (0, 5));
        assert_eq!(a.data, 0b00011101);
        assert_eq!(res, Ok(()));
    }
    #[test]
    fn load_3() {
        let mut a = Bus {
            data: 0b10110111,
            size: 8,
        };
        let b = Bus {
            data: 0b11011101,
            size: 8,
        };
        let res = a.load(&b, (3, 7), (1, 5));
        assert_eq!(a.data, 0b10110111);
        assert_eq!(res, Ok(()));
    }
    #[test]
    fn load_4() {
        let mut a = Bus {
            data: 0b10101011,
            size: 8,
        };
        let b = Bus {
            data: 0b11011100,
            size: 8,
        };
        let res = a.load(&b, (1, 2), (0, 1));
        assert_eq!(a.data, 0b10101010);
        assert_eq!(res, Ok(()));
    }
    #[test]
    fn load_5() {
        let mut a = Bus {
            data: 0b10101011,
            size: 8,
        };
        let b = Bus {
            data: 0b11011100,
            size: 8,
        };
        let res = a.load(&b, (1, 130), (0, 129));
        assert_eq!(res, Err(BusError::SourceBusMaxExceeded));
    }

    #[test]
    fn load_6() {
        let mut a = Bus {
            data: 0b10101011,
            size: 8,
        };
        let b = Bus {
            data: 0b11011100,
            size: 8,
        };
        let res = a.load(&b, (0, 129), (1, 130));
        assert_eq!(res, Err(BusError::DestinationBusMaxExceeded));
    }

    #[test]
    fn pin_0() {
        let a = Bus {
            data: 0b00100000,
            size: 8,
        };
        let b = a.pin(5);
        assert_eq!(b, Bus { data: 1, size: 1 });
    }

    #[test]
    fn pin_1() {
        let a = Bus {
            data: u128::MAX - u128::pow(2, 127),
            size: 128,
        };
        let b = a.pin(127);
        assert_eq!(b, Bus { data: 0, size: 1 });
    }
}
