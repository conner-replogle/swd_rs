# SWD Implementation for the ESP32 in Rust

```rs
let peripherals = Peripherals::take().unwrap();


let mut swd = SwdDriver::new(peripherals.pins.gpio1.into(),peripherals.pins.gpio2.into(),peripherals.pins.gpio4.into());
if let Ok(idcode) = swd.initalize(){
    info!("Connected to SWD -> 0x{:x}",idcode);
    match swd.read_word(0x08000000){
        Ok(mut data) => {
            info!("Read at Option Bytes -> 0x{data:x}");
            let a:u32 = 0xff0000a5;
            
            match swd.write_word(0x08000000, 0xff0000a5){
                Ok(_) => {
                    info!("Writing {:x} to Option Bytes",data);
                    match swd.read_word(0x08000000){
                        Ok( out) => {
                            if out == data {
                                info!("Successfully Wrote at Option Bytes -> {:x}",out);
                            }else{
                                error!("Verify Read returned the wrong Option Bytes -> {:x} instead of {:x}",out,data);

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
    error!("Failed To Connect to write Option Bytes");
}
```