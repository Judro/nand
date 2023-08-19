/// can contain 1 to 127 pins: A[0] .. A[0..128]
struct Bus {
    pub data: u128,
    pub size: u8,
}

fn mask(slice: (u8, u8)) -> u128 {
    (u128::pow(2, slice.1.into()) - 1) ^ (u128::pow(2, slice.0.into()) - 1)
}

impl Bus {
    pub fn load(&mut self, source_bus: &Bus, source: (u8, u8), dest: (u8, u8)) {
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
        assert_eq!(0b0000, mask((0, 0)));
        // A Bus can only contain 127 Pins
        assert_eq!(u128::pow(2, 127) - 1, mask((0, 127)));
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
        a.load(&b, (2, 7), (0, 5));
        assert_eq!(a.data, 0b10111);
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
        a.load(&b, (0, 5), (2, 7));
        assert_eq!(a.data, 0b01110011);
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
        a.load(&b, (0, 5), (0, 5));
        assert_eq!(a.data, 0b00011101);
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
        a.load(&b, (3, 7), (1, 5));
        assert_eq!(a.data, 0b10110111);
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
        a.load(&b, (1, 2), (0, 1));
        assert_eq!(a.data, 0b10101010);
    }
}
