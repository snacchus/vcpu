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

impl From<i32> for Register {
    fn from(v: i32) -> Register {
        Register { i: v }
    }
}

impl From<u32> for Register {
    fn from(v: u32) -> Register {
        Register { u: v }
    }
}

impl From<f32> for Register {
    fn from(v: f32) -> Register {
        Register { f: v }
    }
}

impl PartialEq for Register {
    fn eq(&self, rhs: &Register) -> bool {
        self.u() == rhs.u()
    }
}

impl std::fmt::Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Register")
            .field("i", &self.i())
            .field("u", &self.u())
            .field("f", &self.f())
            .finish()
    }
}
