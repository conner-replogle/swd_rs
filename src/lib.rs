
pub mod constant;
pub mod swd_driver;
pub mod dap;
pub mod error;

#[cfg(test)]
mod tests {
    use crate::swd_driver::SwdDriver;

    use super::*;

    #[test]
    fn it_works() {
        //SwdDriver::new(swdio, swclk, rst)

    }
}
