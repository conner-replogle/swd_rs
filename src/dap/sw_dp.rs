use std::arch::asm;
use std::marker::PhantomData;

use embedded_hal::{digital::v2::OutputPin, can::Error};
use embedded_hal::digital::v2::InputPin;

use crate::config::SwdConfig;
use crate::{constant::*, error::SwdError};
#[no_mangle]
fn noop() -> u32{
    return 20;
}



pub struct Swd<IO,CLK,RST,PinError: std::error::Error>{
    config: SwdConfig,
    swdio:  IO ,
    swclk: CLK,
    rst: Option<RST>,
    _err: PhantomData<PinError>
}
impl <IO,CLK,RST,PinError: std::error::Error>Swd<IO,CLK,RST,PinError>where
IO: OutputPin<Error = PinError>+ InputPin<Error = PinError>,
CLK: OutputPin<Error = PinError>,
RST: OutputPin<Error = PinError>
{
    pub fn new(swdio: IO,swclk: CLK,rst: RST,config:SwdConfig) -> Self{
        Self { config,swdio, swclk, rst:Some(rst), _err: PhantomData }
    }
   
    pub fn setup(&mut self) -> Result<(),SwdError<PinError>>{
        self.swdio.set_high()?;
        self.swclk.set_high()?;

        if self.rst.is_some(){
            self.rst.as_mut().unwrap().set_high()?;
        }  
        Ok(())
    }
    pub fn shutdown(&mut self) ->Result<(),SwdError<PinError>>{
        
        if self.rst.is_some(){
            self.rst.as_mut().unwrap().set_low()?;
        }   
        Ok(()) 

    }
}
impl <IO,CLK,RST,PinError: std::error::Error>Swd<IO,CLK,RST,PinError>where
IO: OutputPin<Error = PinError> + InputPin<Error = PinError>,
CLK: OutputPin<Error = PinError>,
RST: OutputPin<Error = PinError>
{

    #[no_mangle]
    pub fn clock_delay(&self){
        let cycles = ((self.config.clock_speed_mhz*1000 ) as f32/(self.config.target_speed_khz  as f32*30.0))*100.0;
        //TODO Do delay
        for _ in 0..(cycles as u32){
            noop();

        }
    }

    pub fn sw_clock_cycle(&mut self) -> Result<(),SwdError<PinError>>{
        self.swclk.set_low()?;
        self.clock_delay();
        self.swclk.set_high()?;
        self.clock_delay();
        Ok(())
    }

    pub fn sw_write_bit(&mut self,value:bool) -> Result<(),SwdError<PinError>>{
        self.swclk.set_low()?;
        self.swdio.set_state(value.into())?;
        self.clock_delay();
        self.swclk.set_high()?;
        self.clock_delay();
        Ok(())
  
    }

    pub fn sw_read_bit(&mut self) -> Result<bool,SwdError<PinError>>{
        let  value ;
        self.swclk.set_low()?;
        self.clock_delay();
        value = self.swdio.is_high()?;
        self.swclk.set_high()?;
        self.clock_delay();
        return Ok(value);
    }

    pub fn swj_sequence(&mut self,mut count:u32,data: &mut [u8]) -> Result<(),SwdError<PinError>>{
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
                self.swdio.set_high()?;
            } else {
                self.swdio.set_low()?;
                
            } 
            self.sw_clock_cycle()?;
            val >>= 1;
            n-=1;
            count -=1;
        }
        Ok(())
    }
    pub fn swd_transfer(&mut self,request:u8, data: &mut Vec<u8>) -> Result<(),SwdError<PinError>>{
        let mut ack;
        let mut bit;
        let mut val:u32;
        let mut parity:u32;
        
        parity = 0u32;
        self.sw_write_bit(true)?;
        bit = request >> 0;
        self.sw_write_bit(bit & 1 != 0)?;
        parity += bit as u32;
        bit = request >> 1;
        self.sw_write_bit(bit & 1 != 0)?;
        parity += bit as u32;
        bit = request >> 2;
        self.sw_write_bit(bit & 1 != 0)?;
        parity += bit as u32;
        bit = request >> 3;
        self.sw_write_bit(bit & 1 != 0)?;
        parity += bit as u32;
        self.sw_write_bit(parity & 1 != 0)?;
        self.sw_write_bit(false)?;
        self.sw_write_bit(true)?;
        
        //Disable swdio output
        //turnaround
        for _ in 0..self.config.turnaround{
            self.sw_clock_cycle()?;
        }

        bit = self.sw_read_bit()? as u8 ;
        ack = bit << 0;
        bit = self.sw_read_bit()? as u8;
        ack |= bit << 1;
        bit = self.sw_read_bit()? as u8;
        ack |= bit << 2;

        let mut ack = DapResponse::try_from(ack as u8).unwrap_or(DapResponse::DapUnknown(ack as u8));
        if ack == DapResponse::DapTransferOk {         /* OK response */                        
            /* Data transfer */                                                          
            if request & DapTransfer::RnW as u8 != 0 {                                            
                /* Read data */                                                            
                val = 0;                                                                  
                parity = 0;                                                               
                for _ in 0..32 {                                                    
                    bit = self.sw_read_bit()? as u8;              /* Read RDATA[0:31] */                   
                    parity += bit as u32;                                                           
                    val >>= 1;                                                               
                    val |= (bit as u32) << 31;                                                       
                }                                                                          
                bit = self.sw_read_bit()? as u8;                   /* Read Parity */                        
                if (parity ^ (bit as u32)) & 1 != 0 {                                                 
                    ack = DapResponse::DapTransferError.into();                                                
                }                                                                          
                *data = val.to_le_bytes().to_vec();                                                 
                /* Turnaround */                                                           
                for _ in 0..self.config.turnaround{
                    self.sw_clock_cycle()?;
                }                                                                       
               //enable swdio output     
            }else {                                                                     
                /* Turnaround */                                                           
                for _ in 0..self.config.turnaround{
                    self.sw_clock_cycle()?;
                }                                                                          
                //enable swdio output                                                
                /* Write data */                                                           
                val = u32::from_le_bytes(data[0..4].try_into().unwrap());                                                               
                parity = 0;                                                               
                for _ in 0..32 {                                                 
                    self.sw_write_bit(val & 1 != 0)?;         /* Write WDATA[0:31] */                  
                    if val & 1 != 0{
                        parity += 1;
                    }
                    val >>= 1;                                                               
                }                                                             
                self.sw_write_bit(parity % 2 != 0)?;            /* Write Parity Bit */                   
            }                                                                            
            /* Idle cycles */                                                            
            let n = self.config.idle_cycles;                                           
            if n!= 0 {                                                                     
                self.swdio.set_low().unwrap();                                                     
                for _ in 0..n {                                                           
                    self.sw_clock_cycle()?;                                                    
                }                                                                          
            }                                                                            
            self.swdio.set_high().unwrap();                                                             
            match ack{
                DapResponse::DapTransferOk => { return Ok(())},
                _ => {
                    return Err(SwdError::DapResponse(ack));
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
                self.sw_clock_cycle()?;
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
                    return Err(SwdError::DapResponse(ack));
                }
            }                                                       
        }                                                                              
                                                                                        
        /* Protocol error */                                                           
        for _ in 0..(self.config.turnaround+32+1){
            self.sw_clock_cycle()?;
        }                                                                           
        self.swdio.set_high()?;                                                                 

        match ack{
            DapResponse::DapTransferOk => { return Ok(())},
            _ => {
                return Err(SwdError::DapResponse(ack));
            }
        }
    
    }
}
