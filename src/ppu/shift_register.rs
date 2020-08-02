

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
