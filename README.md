# SWD Implementation for the ESP32 in Rust

## Introduction
This is a work in progress crate to implement the SWD protocal in rust, in order to debug MCU's that use SWD from another embedded device. 

## Checklist
- [x] Successfully implement connection sequence.
- [x] Implement AP Register Reading
- [x] Implement DP Register Reading
- [x] Setup error handling
- [x] Read words from memory 
- [ ] write words from memory 
- [ ] read byte from memory 
- [ ] write byte from memory 
- [x] basic clock delay based on cpu speed.
## Example
```rs
let peripherals = Peripherals::take().unwrap();
const SWD_CONFIG: SwdConfig = SwdConfig{
    clock_speed_mhz: 160,
    turnaround: 1,
    idle_cycles:0,
    target_speed_khz: 1000
        
};
//Must be od
let swdio = PinDriver::input_output_od(peripherals.pins.gpio1)?;

let swclk = PinDriver::output(peripherals.pins.gpio2)?;

let rst = PinDriver::output(peripherals.pins.gpio4)?;

let swd = Swd::new(swdio, swclk, rst,SWD_CONFIG);
let mut swd = SwdDriver::new(swd);

if let Ok(idcode) = swd.initalize(){
    info!("Connected to SWD -> 0x{:x}",idcode);
    match swd.read_word(0x08000000){
        Ok(mut data) => {
            info!("Read at 0x08000000 -> 0x{data:x}");
            let a:u32 = 0xff0000a5;
            
            match swd.write_word(0x08000000, 0xff0000a5){
                Ok(_) => {
                    info!("Writing {:x} to 0x08000000",data);
                    match swd.read_word(0x08000000){
                        Ok( out) => {
                            if out == data {
                                info!("Successfully Wrote at 0x08000000 -> {:x}",out);
                            }else{
                                error!("Verify Read returned the wrong 0x08000000 -> {:x} instead of {:x}",out,data);

                            }
                        },
                        Err(response) => {
                            error!("Failed verify read {response:?}");
                        }
                    }
                    
                },
                Err(response) => {
                    error!("Failed write {response:?}");
                }
            }
        },
        Err(response) => {
            error!("Failed Read {response:?}");
        }
    }
}else{
    error!("Failed To Connect");
}
```