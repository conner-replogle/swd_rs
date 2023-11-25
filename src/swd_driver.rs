use std::cell::Ref;


use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin};
use esp_idf_sys::tm;

use crate::{dap::sw_dp::Swd, constant::{DapResponse, MAX_SWD_RETRY, DpRegister, Register, AbortRegisterDef, DebugControlStatusDef, DebugSelectRegister, AccessPortRegisterAddress, APControlStatusWordDef, CSW_VALUE}};

pub struct SwdDriver<'a>{
    pub swd: Swd<'a>

}
impl <'a>SwdDriver<'a>{
    pub fn new(swdio: AnyIOPin,swclk: AnyOutputPin,rst: AnyOutputPin) -> Self{
        Self { 
            swd: Swd::new(swdio, swclk, rst)
        }
    }
    pub fn connect(&mut self) -> Result<u32,DapResponse>{
        self.swd.setup();
        self.reset();
    
        self.swd_switch(0xE79E);
    
        self.reset();
    
        return self.swd_read_idcode();
    }
    pub fn initalize(&mut self) -> Result<u32,DapResponse>{
        let code = self.connect()?;
        self.write_dp(DpRegister::ABORT, AbortRegisterDef::STKCMPCLR | AbortRegisterDef::STKERRCLR | AbortRegisterDef::WDERRCLR | AbortRegisterDef::ORUNERRCLR)?;
        self.write_dp(DpRegister::SELECT, 0)?;
        self.write_dp(DpRegister::CTRL_STAT, DebugControlStatusDef::CSYSPWRUPREQ | DebugControlStatusDef::CDBGPWRUPREQ)?;
        for _ in 0..100 {
            match self.read_dp(DpRegister::CTRL_STAT)?{
                val if val & (DebugControlStatusDef::CSYSPWRUPACK | DebugControlStatusDef::CDBGPWRUPACK) == (DebugControlStatusDef::CSYSPWRUPACK | DebugControlStatusDef::CDBGPWRUPACK) => {
                    break;
                },
                _ => {}
            }
        }
        self.write_dp(DpRegister::CTRL_STAT, DebugControlStatusDef::CSYSPWRUPREQ | DebugControlStatusDef::CDBGPWRUPREQ | DebugControlStatusDef::TRNNORMAL | DebugControlStatusDef::MASKLANE)?;
        self.write_dp(DpRegister::SELECT, 0)?;
        Ok(code)
    }

    pub fn reset(&mut self){
        let mut tmp_in= [0xffu8;8];
        self.swd.swj_sequence(51,&mut tmp_in);
    }
    pub fn swd_switch(&mut self,val:u16){
        let mut tmp_in= [0x00u8;2];

        tmp_in[0] = (val  & 0xff) as u8;
        tmp_in[1] = ((val >> 8) & 0xff) as u8;
        self.swd.swj_sequence(16,&mut tmp_in);
    }
    pub fn swd_read_idcode(&mut self) -> Result<u32,DapResponse>
    {
        let mut tmp_in = [0u8;1];

        self.swd.swj_sequence(8,&mut tmp_in);
        return self.read_dp(0);
    }
    pub fn swd_transfer_retry(&mut self,req:u8,data: &mut Vec<u8>) -> Result<(),DapResponse>{
        let mut ack = Err(DapResponse::DapTransferWait);

        for _ in 0..MAX_SWD_RETRY {
            ack = self.swd.swd_transfer(req, data);
            match ack{
                Err(DapResponse::DapTransferWait) => {continue;},
                ack => {
                    return ack;
                }
            };
     
        }

        return ack;
    }

    pub fn read_dp(&mut self,adr: u8) -> Result<u32,DapResponse>{
        let mut tmp_out = Vec::with_capacity(4);

        let tmp_in = Register::DP | Register::R | Register::ADR(adr) ;
        let ack = self.swd_transfer_retry(tmp_in, &mut tmp_out)?;
        
        return Ok(u32::from_le_bytes(tmp_out[0..4].try_into().unwrap()));
    }
    pub fn read_ap(&mut self,adr: u8,) -> Result<u32,DapResponse>{
        let mut tmp_out = Vec::with_capacity(4);

        let tmp_in = Register::AP | Register::R | Register::ADR(adr as u8) ;
        let ack = self.swd_transfer_retry(tmp_in, &mut tmp_out)?;
       
    
        let apsel:u32 = (adr as u32) & 0xff000000;
        let bank_sel = (adr as u32 & DebugSelectRegister::APBANKSEL) as u32;

        self.write_dp(DpRegister::SELECT, apsel | bank_sel)?;

    // first dummy read
        self.swd_transfer_retry(tmp_in, &mut tmp_out)?;
        self.swd_transfer_retry(tmp_in, &mut tmp_out)?;
        
        return Ok(u32::from_le_bytes(tmp_out[0..4].try_into().unwrap()));
    }
    pub fn write_dp(&mut self, adr: u8,val:u32) -> Result<(),DapResponse>{
        let req = Register::DP | Register::W | Register::ADR(adr );
        // switch (adr) {
        //     case DP_SELECT:
        //         if (dap_state.select == val) {
        //             return 1;
        //         }
    
        //         dap_state.select = val;
        //         break;
    
        //     default:
        //         break;
        // }
        let mut data :Vec<u8> = Vec::new();
        data = val.to_le_bytes().to_vec();
        let ack = self.swd.swd_transfer(req, &mut data);
        return ack;
    }
    
// Write access port register
    fn write_ap(&mut self,adr:u8, val:u32) -> Result<(),DapResponse>{
    
        let apsel = (adr as u32) & 0xff000000;
        let bank_sel = (adr as u32) & DebugSelectRegister::APBANKSEL;

        self.write_dp(DpRegister::SELECT, apsel | bank_sel);

        //some type of caching
        // switch (adr) {
        //     case AP_CSW:
        //         if (dap_state.csw == val) {
        //             return 1;
        //         }

        //         dap_state.csw = val;
        //         break;

        //     default:
        //         break;
        // }

        let req = Register::AP | Register::W | Register::ADR(adr);
        let mut data = val.to_le_bytes().to_vec();


        self.swd_transfer_retry(req, &mut data)?;
        

        let req = Register::DP | Register::R | Register::ADR(DpRegister::RDBUFF);
        self.swd_transfer_retry(req, &mut Vec::new())?;
        
        return Ok(());
    }

    // Rewrite the following cpp code into rust
    // Read 32-bit word from target memory.
    pub fn read_word(&mut self,addr:u32) -> Result<u32,DapResponse>
    {
        self.write_ap(AccessPortRegisterAddress::AP_CSW, CSW_VALUE | APControlStatusWordDef::CSW_SIZE32)?;
        return self.read_data(addr);
    }

    // Write 32-bit word to target memory.
    pub fn write_word(&mut self,addr:u32,val:u32) -> Result<(),DapResponse>
    {
        self.write_ap(AccessPortRegisterAddress::AP_CSW, CSW_VALUE | APControlStatusWordDef::CSW_SIZE32)?;

        self.write_data(addr, val)?;


        return Ok(());
    }

    // // Read 8-bit byte from target memory.
    // static uint8_t swd_read_byte(uint32_t addr, uint8_t *val)
    // {
    //     uint32_t tmp;

    //     if (!swd_write_ap(AP_CSW, CSW_VALUE | CSW_SIZE8)) {
    //         return 0;
    //     }

    //     if (!swd_read_data(addr, &tmp)) {
    //         return 0;
    //     }

    //     *val = (uint8_t)(tmp >> ((addr & 0x03) << 3));
    //     return 1;
    // }

    // // Write 8-bit byte to target memory.
    // static uint8_t swd_write_byte(uint32_t addr, uint8_t val)
    // {
    //     uint32_t tmp;

    //     if (!swd_write_ap(AP_CSW, CSW_VALUE | CSW_SIZE8)) {
    //         return 0;
    //     }

    //     tmp = val << ((addr & 0x03) << 3);

    //     if (!swd_write_data(addr, tmp)) {
    //         return 0;
    //     }

    //     return 1;
    // }
    fn read_data(&mut self,addr:u32) -> Result<u32,DapResponse>{
        let mut tmp_out: Vec<u8> = Vec::new();
        
        let req = Register::AP | Register::W | (1<<2);
        self.swd_transfer_retry(req, &mut addr.to_le_bytes().to_vec())?;
        let req = Register::AP |  Register::R | (3 << 2);
        self.swd_transfer_retry(req, &mut tmp_out)?;
        let req = Register::DP | Register::R  | Register::ADR(DpRegister::RDBUFF);
        self.swd_transfer_retry(req, &mut tmp_out)?;
        let val = u32::from_le_bytes(tmp_out[0..4].try_into().unwrap());
        return Ok(val);
    }
    fn write_data(&mut self,addr:u32,data:u32) -> Result<(),DapResponse>{
        let mut tmp_in = addr.to_le_bytes().to_vec();
    
        // put addr in TAR register
        let req = Register::AP | Register::W | (1 << 2);

        self.swd_transfer_retry(req, &mut tmp_in)?;
        
        
        let mut tmp_in = data.to_le_bytes().to_vec();
        let req = Register::AP | Register::W | (3 << 2);

        self.swd_transfer_retry(req,   &mut tmp_in)?;

        // dummy read
        let req = Register::DP | Register::R | Register::ADR(DpRegister::RDBUFF);
        self.swd_transfer_retry(req, &mut Vec::new())?;
        return Ok(())
    }
}