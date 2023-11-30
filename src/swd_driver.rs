
use embedded_hal::digital::v2::{OutputPin, InputPin};

use crate::{dap::sw_dp::Swd, constant::{DapResponse, MAX_SWD_RETRY, DpRegister::{self, RDBUFF}, Register, AbortRegisterDef, DebugControlStatusDef, DebugSelectRegister, AccessPortRegisterAddress, APControlStatusWordDef, CSW_VALUE}, error::SwdError};
/// SWD Driver takes in a io pin and output pin and a reset pin
pub struct SwdDriver<IO,CLK,RST,PinError: std::error::Error>{
    pub swd: Swd<IO,CLK,RST,PinError>

}
impl <IO,CLK,RST,PinError: std::error::Error>SwdDriver<IO,CLK,RST,PinError> where
IO: OutputPin<Error = PinError> + InputPin<Error = PinError>,
CLK: OutputPin<Error = PinError>,
RST: OutputPin<Error = PinError>
{
    pub fn new(swd: Swd<IO,CLK,RST,PinError>) -> Self{
        Self { 
            swd,
        }
    }
    fn connect(&mut self) -> Result<u32,SwdError<PinError>>{
        self.swd.setup()?;
        self.reset()?;
    
        self.switch(0xE79E)?;
    
        self.reset()?;
    
        return self.read_idcode();
    }
    pub fn initalize(&mut self) -> Result<u32,SwdError<PinError>>{
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

    pub fn reset(&mut self) -> Result<(),SwdError<PinError>>{
        let mut tmp_in= [0xffu8;8];
        self.swd.swj_sequence(51,&mut tmp_in)?;
        Ok(())
    }
    pub fn switch(&mut self,val:u16)-> Result<(),SwdError<PinError>>{
        let mut tmp_in= [0x00u8;2];

        tmp_in[0] = (val  & 0xff) as u8;
        tmp_in[1] = ((val >> 8) & 0xff) as u8;
        self.swd.swj_sequence(16,&mut tmp_in)?;
        Ok(())
    }
    pub fn read_idcode(&mut self) -> Result<u32,SwdError<PinError>>
    {
        let mut tmp_in = [0u8;1];

        self.swd.swj_sequence(8,&mut tmp_in)?;
        return self.read_dp(0);
    }
    fn transfer_retry(&mut self,req:u8,data: &mut [u8;4]) -> Result<(),SwdError<PinError>>{
        let mut ack = Err( SwdError::DapResponse(DapResponse::DapTransferWait));

        for _ in 0..MAX_SWD_RETRY {
            ack = self.swd.swd_transfer(req, data);
            match ack{
                Err(SwdError::DapResponse(DapResponse::DapTransferWait)) => {continue;},
                ack => {
                    return ack;
                }
            };
        }

        return ack;
    }
    #[cfg(feature = "glitch")]
    fn transfer_glitch<F: FnMut() -> ()>(&mut self,req:u8,data: &mut [u8;4],glitch: F) -> Result<(),SwdError<PinError>>{
        return self.swd.glitch_swd_transfer(req, data,glitch);
    }

    pub fn read_dp(&mut self,adr: u8) -> Result<u32,SwdError<PinError>>{
        let mut tmp_out = [0u8;4];

        let tmp_in = Register::DP | Register::R | Register::ADR(adr) ;
        self.transfer_retry(tmp_in, &mut tmp_out)?;
        
        return Ok(u32::from_le_bytes(tmp_out[0..4].try_into().unwrap()));
    }
    pub fn read_ap(&mut self,adr: u8,) -> Result<u32,SwdError<PinError>>{
        let mut tmp_out = [0u8;4];

        let tmp_in = Register::AP | Register::R | Register::ADR(adr as u8) ;
        self.transfer_retry(tmp_in, &mut tmp_out)?;
       
    
        let apsel:u32 = (adr as u32) & 0xff000000;
        let bank_sel = (adr as u32 & DebugSelectRegister::APBANKSEL) as u32;

        self.write_dp(DpRegister::SELECT, apsel | bank_sel)?;

    // first dummy read
        self.transfer_retry(tmp_in, &mut tmp_out)?;
        self.transfer_retry(tmp_in, &mut tmp_out)?;
        
        return Ok(u32::from_le_bytes(tmp_out[0..4].try_into().unwrap()));
    }
    pub fn write_dp(&mut self, adr: u8,val:u32) -> Result<(),SwdError<PinError>>{
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
        let mut data = val.to_le_bytes();
        let ack = self.swd.swd_transfer(req, &mut data);
        return ack;
    }
    
// Write access port register
    pub fn write_ap(&mut self,adr:u8, val:u32) -> Result<(),SwdError<PinError>>{
    
        let apsel = (adr as u32) & 0xff000000;
        let bank_sel = (adr as u32) & DebugSelectRegister::APBANKSEL;

        self.write_dp(DpRegister::SELECT, apsel | bank_sel)?;

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
        let mut data = val.to_le_bytes();


        self.transfer_retry(req, &mut data)?;
        
         //   TODO MAybe dont remove
        let req = Register::DP | Register::R | Register::ADR(DpRegister::RDBUFF);
        self.transfer_retry(req, &mut [0u8;4])?;
        
        return Ok(());
    }

    // Rewrite the following cpp code into rust
    // Read 32-bit word from target memory.
    pub fn read_word(&mut self,addr:u32) -> Result<u32,SwdError<PinError>>
    {
        self.write_ap(AccessPortRegisterAddress::AP_CSW, CSW_VALUE | APControlStatusWordDef::CSW_SIZE32)?;
        return self.read_data(addr);
    }
    #[cfg(feature = "glitch")]
    pub fn read_word_glitched<F: FnMut() -> ()>(&mut self,addr:u32,glitch: F) -> Result<u32,SwdError<PinError>>
    {
        self.write_ap(AccessPortRegisterAddress::AP_CSW, CSW_VALUE | APControlStatusWordDef::CSW_SIZE32)?;
        return self.read_data_glitch(addr,glitch);
    }

    // Write 32-bit word to target memory.
    pub fn write_word(&mut self,addr:u32,val:u32) -> Result<(),SwdError<PinError>>
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
    fn read_data(&mut self,addr:u32) -> Result<u32,SwdError<PinError>>{
        let mut tmp_out = [0u8;4];
        
        let req = Register::AP | Register::W | (1<<2);
        self.transfer_retry(req, &mut addr.to_le_bytes())?;
        let req = Register::AP |  Register::R | (3 << 2);

        self.transfer_retry(req, &mut tmp_out)?;

        let req = Register::DP | Register::R  | Register::ADR(DpRegister::RDBUFF);
        self.transfer_retry(req, &mut tmp_out)?;
        let val = u32::from_le_bytes(tmp_out[0..4].try_into().unwrap());
        return Ok(val);
    }
    #[cfg(feature = "glitch")]
    fn read_data_glitch<F: FnMut() -> ()>(&mut self,addr:u32,glitch: F) -> Result<u32,SwdError<PinError>>{
        let mut tmp_out= [0u8;4];
        
        let req = Register::AP | Register::W | (1<<2);
        self.transfer_retry(req, &mut addr.to_le_bytes())?;
        let req = Register::AP |  Register::R | (3 << 2);

        self.transfer_retry(req, &mut tmp_out)?;

        let req = Register::DP | Register::R  | Register::ADR(DpRegister::RDBUFF);
        self.transfer_glitch(req, &mut tmp_out,glitch)?;
        let val = u32::from_le_bytes(tmp_out[0..4].try_into().unwrap());
        return Ok(val);
    }
    // Write 32-bit word aligned values to target memory using address auto-increment.
// size is in bytes.
    pub fn write_block(&mut self,address:u32, data: &mut Vec<u8>)-> Result<(),SwdError<PinError>>
    {
     
        let size = data.len();

        if size == 0 {
            return Err(SwdError::IncorrectParams);
        }


        // CSW register
        self.write_ap(AccessPortRegisterAddress::AP_CSW, CSW_VALUE | APControlStatusWordDef::CSW_SIZE32)?;

        // TAR write
        let req = Register::AP | Register::W | (1 << 2);
        let mut tmp_in = address.to_le_bytes();

        self.transfer_retry(req, &mut tmp_in)?;

        // DRW write
        let req = Register::AP | Register::W | (3 << 2);

        for slice in data.chunks_exact_mut(4) {
            self.transfer_retry(req, slice.try_into().expect("size of 4"))?;
        }

        // dummy read
        let req = Register::DP | Register::R | Register::ADR(RDBUFF);
        return self.transfer_retry(req, &mut [0u8;4]);
    
    }
    fn write_data(&mut self,addr:u32,data:u32) -> Result<(),SwdError<PinError>>{
        let mut tmp_in = addr.to_le_bytes();
    
        // put addr in TAR register
        let req = Register::AP | Register::W | (1 << 2);

        self.transfer_retry(req, &mut tmp_in)?;
        
        
        let mut tmp_in = data.to_le_bytes();
        let req = Register::AP | Register::W | (3 << 2);

        self.transfer_retry(req,   &mut tmp_in)?;

        // dummy read
        let req = Register::DP | Register::R | Register::ADR(DpRegister::RDBUFF);
        self.transfer_retry(req, &mut [0u8;4])?;
        return Ok(())
    }
}