

#[derive(Default)]
pub struct ShiftRegister {
    inner_vec: Vec<u8>
}

impl ShiftRegister {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        Self { inner_vec: vec![0; size] }
    }

    pub fn get(&self) -> u8 {
        self.inner_vec.first().expect("ShiftRegister should never be empty").clone()
    }

    pub fn set(&mut self, value: u8) {
        self.inner_vec.last_mut().map(|x| *x = value);
    }

    pub fn shift_bytes(&mut self) {
        for i in 1 .. self.inner_vec.len() {
            self.inner_vec[i-1] = self.inner_vec[i];
        }
        self.inner_vec.last_mut().map(|x| *x = 0);
    }

    pub fn shift_bits(&mut self) {
        for i in 1 .. self.inner_vec.len() {
            self.inner_vec[i-1] >>= 1;
            self.inner_vec[i-1] |= (self.inner_vec[i] & 1) << 7;
        }
        self.inner_vec.last_mut().map(|x| *x >>= 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_register_get_set() -> Result<(), std::io::Error> {
        let mut a = ShiftRegister::new(2);
        assert!(a.get() == 0);
        a.set(0x56);
        assert!(a.get() == 0);
        a.set(0xfd);
        assert!(a.get() == 0);
        Ok(())
    }

    #[test]
    fn test_shift_register_shift_bytes() -> Result<(), std::io::Error> {
        let mut a = ShiftRegister::new(2);
        assert!(a.get() == 0);
        a.set(0x56);
        a.shift_bytes();
        assert!(a.get() == 0x56);
        a.shift_bytes();
        assert!(a.get() == 0x00);
        a.shift_bytes();
        assert!(a.get() == 0x00);
        a.shift_bytes();
        assert!(a.get() == 0x00);
        Ok(())
    }

    #[test]
    fn test_shift_register_shift_bits() -> Result<(), std::io::Error> {
        let mut a = ShiftRegister::new(2);
        assert!(a.get() == 0);
        a.set(0b000_0101);
        assert!(a.get() == 0);
        a.shift_bits();
        let mut b = 0b1000_0000;
        assert!(a.get() == b);
        a.shift_bits();
        b >>= 1;
        assert!(a.get() == b);
        a.shift_bits();
        b = 0b1010_0000;
        assert!(a.get() == b);
        for _ in 0..20 {
            a.shift_bits();
            b >>= 1;
            assert!(a.get() == b);
        }
        Ok(())
    }
}
