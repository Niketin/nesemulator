
#[derive(Default)]
pub struct ShiftRegister<T: Copy + Default> {
    inner_array: [T; 3]
}

impl<T: Copy + Default> ShiftRegister<T> {
    pub fn get(&self) -> T {
        self.inner_array[0]
    }

    pub fn set(&mut self, value: T) {
        self.inner_array[2] = value;
    }

    pub fn shift(&mut self) {
        self.inner_array[0] = self.inner_array[1];
        self.inner_array[1] = self.inner_array[2];
        self.inner_array[2] = T::default();
    }
}
