use esp_idf_hal::{gpio::{IOPin, InputOutput, PinDriver, OutputPin, Output, AnyIOPin, AnyOutputPin, Level}, delay::ets_delay_us};
use std::arch::asm;

use crate::constant::*;


pub fn pin_delay(){

}


pub struct SwdConfig{
    clock_delay: u32,
    turnaround:u32 ,
    idle_cycles:u32,
}
impl Default for SwdConfig{
    fn default() -> Self{
        Self { clock_delay: CLOCK_DELAY ,turnaround: 1,idle_cycles:0}
    }
}
pub struct Swd<'a>{
    config: SwdConfig,
    swdio: PinDriver<'a,AnyIOPin,InputOutput>,
    swclk: PinDriver<'a,AnyOutputPin,Output>,
    rst: PinDriver<'a,AnyOutputPin,Output>,
}
impl <'a>Swd<'a>{
    pub fn new(swdio: AnyIOPin,swclk: AnyOutputPin,rst: AnyOutputPin) -> Self{
        Self { config: SwdConfig::default(),swdio:PinDriver::input_output_od(swdio).unwrap(), swclk: PinDriver::output(swclk).unwrap(), rst: PinDriver::output(rst).unwrap() }
    }
    pub fn setup(&mut self){
        self.swdio.set_high().unwrap();
        self.swclk.set_high().unwrap();
        self.rst.set_high().unwrap();   

    }
    pub fn shutdown(&mut self){
        
        self.rst.set_low().unwrap();   

    }
}
impl <'a>Swd<'a>{


    pub fn clock_delay(&self){
        let delay =self.config.clock_delay * ((CPU_CLOCK/1000u32) + (DELAY_SLOW_CYCLES-1u32)) / DELAY_SLOW_CYCLES;
        for _ in 0..500{
        unsafe{
            asm!("nop");
        }
        }
    }
    #[inline]
    pub fn sw_clock_cycle(&mut self){
        self.swclk.set_low().unwrap();
        self.clock_delay();
        self.swclk.set_high().unwrap();
        self.clock_delay();
    }
    #[inline]
    pub fn sw_write_bit(&mut self,value:bool){
        self.swclk.set_low().unwrap();
        self.swdio.set_level(value.into()).unwrap();
        
        self.clock_delay();
        self.swclk.set_high().unwrap();
        self.clock_delay();
  
    }
    #[inline]
    pub fn sw_read_bit(&mut self) -> bool{
        let  value ;
        self.swclk.set_low().unwrap();
        self.clock_delay();
        value = self.swdio.get_level().into();
        self.swclk.set_high().unwrap();
        self.clock_delay();
        return value;
    }

    pub fn swj_sequence(&mut self,mut count:u32,data: &mut [u8]){
        let mut val = 0u8;
        let mut n = 0u32;
        let mut index = 0;
        while count> 0 {
            if n == 0u32 {
                
                val = data[index];
                index +=1;
                n = 8;
            }
            if val & 1 != 0 {
                self.swdio.set_high().unwrap();
            } else {
                self.swdio.set_low().unwrap();
                
            } 
            self.sw_clock_cycle();
            val >>= 1;
            n-=1;
            count -=1;
        }
    }
    pub fn swd_transfer(&mut self,request:u8, data: &mut Vec<u8>) -> Result<(),DapResponse>{
        let mut ack;
        let mut bit;
        let mut val:u32;
        let mut parity:u32;
        
        parity = 0u32;
        self.sw_write_bit(true);
        bit = request >> 0;
        self.sw_write_bit(bit & 1 != 0);
        parity += bit as u32;
        bit = request >> 1;
        self.sw_write_bit(bit & 1 != 0);
        parity += bit as u32;
        bit = request >> 2;
        self.sw_write_bit(bit & 1 != 0);
        parity += bit as u32;
        bit = request >> 3;
        self.sw_write_bit(bit & 1 != 0);
        parity += bit as u32;
        self.sw_write_bit(parity & 1 != 0);
        self.sw_write_bit(false);
        self.sw_write_bit(true);
        
        //Disable swdio output
        //turnaround
        for _ in 0..self.config.turnaround{
            self.sw_clock_cycle();
        }

        bit = self.sw_read_bit() as u8 ;
        ack = bit << 0;
        bit = self.sw_read_bit() as u8;
        ack |= bit << 1;
        bit = self.sw_read_bit() as u8;
        ack |= bit << 2;

        let mut ack = DapResponse::try_from(ack as u8).unwrap_or(DapResponse::DapUnknown(ack as u8));
        if ack == DapResponse::DapTransferOk {         /* OK response */                        
            /* Data transfer */                                                          
            if request & DapTransfer::RnW as u8 != 0 {                                            
                /* Read data */                                                            
                val = 0;                                                                  
                parity = 0;                                                               
                for n in 0..32 {                                                    
                    bit = self.sw_read_bit() as u8;              /* Read RDATA[0:31] */                   
                    parity += bit as u32;                                                           
                    val >>= 1;                                                               
                    val |= (bit as u32) << 31;                                                       
                }                                                                          
                bit = self.sw_read_bit() as u8;                   /* Read Parity */                        
                if (parity ^ (bit as u32)) & 1 != 0 {                                                 
                    ack = DapResponse::DapTransferError.into();                                                
                }                                                                          
                *data = val.to_le_bytes().to_vec();                                                 
                /* Turnaround */                                                           
                for _ in 0..self.config.turnaround{
                    self.sw_clock_cycle();
                }                                                                       
               //enable swdio output     
            }else {                                                                     
                /* Turnaround */                                                           
                for _ in 0..self.config.turnaround{
                    self.sw_clock_cycle();
                }                                                                          
                //enable swdio output                                                
                /* Write data */                                                           
                val = u32::from_le_bytes(data[0..4].try_into().unwrap());                                                               
                parity = 0;                                                               
                for _ in 0..32 {                                                 
                    self.sw_write_bit(val & 1 != 0);         /* Write WDATA[0:31] */                  
                    if val & 1 != 0{
                        parity += 1;
                    }
                    val >>= 1;                                                               
                }                                                             
                self.sw_write_bit(parity % 2 != 0);            /* Write Parity Bit */                   
            }                                                                            
            /* Idle cycles */                                                            
            let n = self.config.idle_cycles;                                           
            if n!= 0 {                                                                     
                self.swdio.set_low().unwrap();                                                     
                for _ in 0..n {                                                           
                    self.sw_clock_cycle();                                                    
                }                                                                          
            }                                                                            
            self.swdio.set_high().unwrap();                                                             
            match ack{
                DapResponse::DapTransferOk => { return Ok(())},
                _ => {
                    return Err(ack);
                }
            }                                                     
        }                                                                                                                                                   
        if ack == DapResponse::DapTransferWait || (ack == DapResponse::DapTransferFault) {               
            /* WAIT or FAULT response */                                                 
            // if  request & (DapTransfer::RnW as u8 )!= 0 {  
            //     for _ in 0..33{
            //         self.sw_clock_cycle();

            //     }                                                                          
            // }                                                                            
            /* Turnaround */                                                             
            for _ in 0..self.config.turnaround{
                self.sw_clock_cycle();
            }                                                                             
            //enable swdio out                                                  
            // if  request & (DapTransfer::RnW as u8 )!= 0 {  
            //     self.swdio.set_high().unwrap();
            //     for _ in 0..33{
            //         self.sw_clock_cycle();

            //     }                                                                          
            // }                                                                             
            self.swdio.set_high().unwrap();                                                       
            match ack{
                DapResponse::DapTransferOk => { return Ok(())},
                _ => {
                    return Err(ack);
                }
            }                                                       
        }                                                                              
                                                                                        
        /* Protocol error */                                                           
        for _ in 0..(self.config.turnaround+32+1){
            self.sw_clock_cycle();
        }                                                                           
        self.swdio.set_high().unwrap();                                                                 

        match ack{
            DapResponse::DapTransferOk => { return Ok(())},
            _ => {
                return Err(ack);
            }
        }
    
    }
}
