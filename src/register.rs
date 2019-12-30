#[derive(Clone, Copy)]
pub union Register {
    i: i32,
    u: u32,
    f: f32,
}

impl Register {
    pub fn i(self) -> i32 {
        unsafe { self.i }
    }

    pub fn u(self) -> u32 {
        unsafe { self.u }
    }

    pub fn f(self) -> f32 {
        unsafe { self.f }
    }

    pub fn set_i(&mut self, value: i32) {
        self.i = value;
    }

    pub fn set_u(&mut self, value: u32) {
        self.u = value;
    }

    pub fn set_f(&mut self, value: f32) {
        self.f = value;
    }
}

impl Default for Register {
    fn default() -> Register {
        Register { u: 0 }
    }
}
