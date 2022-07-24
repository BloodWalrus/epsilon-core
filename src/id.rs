static mut CURRENT_ID: u128 = 0;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Id(u128);

impl Id {
    pub fn new() -> Self {
        unsafe {
            CURRENT_ID += 1;
            Self(CURRENT_ID)
        }
    }
}
