


pub const DELAY_SLOW_CYCLES:u32 = 3;
pub const MAX_SWD_RETRY:u32 = 10;
pub const MAX_TIMEOUT:u32 = 1000000;

pub const CSW_VALUE:u32 = (APControlStatusWordDef::CSW_RESERVED | APControlStatusWordDef::CSW_MSTRDBG | APControlStatusWordDef::CSW_HPROT | APControlStatusWordDef::CSW_DBGSTAT | APControlStatusWordDef::CSW_SADDRINC);

#[repr(u8)]
#[derive(PartialEq,Debug)]
pub enum DapResponse
{
    DapUnknown(u8) = 69,
    DapTransferInvalid      = 0,
    DapTransferOk           = 1 << 0,
    DapTransferWait         = 1 << 1,
    DapTransferFault        = 1 << 2,
    DapTransferError        = 1 << 3,
    DapTransferMismatch     = 1 << 4,
}
impl From<u8> for DapResponse{
    fn from(value: u8) -> Self {
        match value{
            0 => DapResponse::DapTransferInvalid,
            1 => DapResponse::DapTransferOk,
            2 => DapResponse::DapTransferWait,
            4 => DapResponse::DapTransferFault,
            8 => DapResponse::DapTransferError,
            16 => DapResponse::DapTransferMismatch,
            val => DapResponse::DapUnknown(val),
        }
    }
}

pub type DapResult = Result<(), DapResponse>;
#[repr(u8)]
#[derive(PartialEq)]
pub enum DapTransfer
{
    ApnDp        = 1 << 0,
    RnW          = 1 << 1,
    A2           = 1 << 2,
    A3           = 1 << 3,
    MatchValue  = 1 << 4,
    MatchMask   = 1 << 5,
}

pub mod DpRegister{
    pub const IDCODE   :u8 =                 0x00;   // IDCODE Register (SW Read only)
    pub const ABORT    :u8 =                 0x00;   // Abort Register (SW Write only)
    pub const CTRL_STAT:u8 =                 0x04;   // Control & Status
    pub const WCR      :u8 =                 0x04;   // Wire Control Register (SW Only)
    pub const SELECT   :u8 =                 0x08;   // Select Register (JTAG R/W & SW W)
    pub const RESEND   :u8 =                 0x08;   // Resend (SW Read Only)
    pub const RDBUFF   :u8 =                 0x0C;   // Read Buffer (Read Only)
}


pub mod Register
{
    pub const AP:u8 = 1;
    pub const DP:u8 = 0;
    pub const R:u8 = 1<<1;
    pub const W:u8 = 0<<1;
    pub const ADR: fn(a: u8) -> u8 =  |a| a & 0x0C;

}
// Abort Register definitions

pub mod AbortRegisterDef{
    pub const DAPABORT  :u32  =   0x00000001;  // DAP Abort
    pub const STKCMPCLR :u32  =   0x00000002;  // Clear STICKYCMP Flag (SW Only)
    pub const STKERRCLR :u32  =   0x00000004;  // Clear STICKYERR Flag (SW Only)
    pub const WDERRCLR  :u32  =   0x00000008;  // Clear WDATAERR Flag (SW Only)
    pub const ORUNERRCLR:u32  =   0x00000010;  // Clear STICKYORUN Flag (SW Only)

}
pub mod DebugSelectRegister{
    pub const CTRLSEL  :u32   =   0x00000001;  // CTRLSEL (SW Only)
    pub const APBANKSEL:u32   =   0x000000F0;  // APBANKSEL Mask
    pub const APSEL    :u32   =   0xFF000000;  // APSEL Mask
}

// Debug Control and Status definitions
pub mod DebugControlStatusDef{
    pub const  ORUNDETECT  :u32 =  0x00000001;  // Overrun Detect
    pub const  STICKYORUN  :u32 =  0x00000002;  // Sticky Overrun
    pub const  TRNMODE     :u32 =  0x0000000C;  // Transfer Mode Mask
    pub const  TRNNORMAL   :u32 =  0x00000000;  // Transfer Mode: Normal
    pub const  TRNVERIFY   :u32 =  0x00000004;  // Transfer Mode: Pushed Verify
    pub const  TRNCOMPARE  :u32 =  0x00000008;  // Transfer Mode: Pushed Compare
    pub const  STICKYCMP   :u32 =  0x00000010;  // Sticky Compare
    pub const  STICKYERR   :u32 =  0x00000020;  // Sticky Error
    pub const  READOK      :u32 =  0x00000040;  // Read OK (SW Only)
    pub const  WDATAERR    :u32 =  0x00000080;  // Write Data Error (SW Only)
    pub const  MASKLANE    :u32 =  0x00000F00;  // Mask Lane Mask
    pub const  MASKLANE0   :u32 =  0x00000100;  // Mask Lane 0
    pub const  MASKLANE1   :u32 =  0x00000200;  // Mask Lane 1
    pub const  MASKLANE2   :u32 =  0x00000400;  // Mask Lane 2
    pub const  MASKLANE3   :u32 =  0x00000800;  // Mask Lane 3
    pub const  TRNCNT      :u32 =  0x001FF000;  // Transaction Counter Mask
    pub const  CDBGRSTREQ  :u32 =  0x04000000;  // Debug Reset Request
    pub const  CDBGRSTACK  :u32 =  0x08000000;  // Debug Reset Acknowledge
    pub const  CDBGPWRUPREQ:u32 =  0x10000000;  // Debug Power-up Request
    pub const  CDBGPWRUPACK:u32 =  0x20000000;  // Debug Power-up Acknowledge
    pub const  CSYSPWRUPREQ:u32 =  0x40000000;  // System Power-up Request
    pub const  CSYSPWRUPACK:u32 =  0x80000000;  // System Power-up Acknowpub const 
}

pub mod AccessPortRegisterAddress{
    pub const AP_CSW  :u8 = 0x00;        // Control and Status Word
    pub const AP_TAR  :u8 = 0x04;        // Transfer Address
    pub const AP_DRW  :u8 = 0x0C;        // Data Read/Write
    pub const AP_BD0  :u8 = 0x10;        // Banked Data 0
    pub const AP_BD1  :u8 = 0x14;        // Banked Data 1
    pub const AP_BD2  :u8 = 0x18;        // Banked Data 2
    pub const AP_BD3  :u8 = 0x1C;        // Banked Data 3
    pub const AP_ROM  :u8 = 0xF8;        // Debug ROM Address
    pub const AP_IDR  :u8 = 0xFC;        // Identification Register
}

/// AP Control and Status Word definitions
pub mod APControlStatusWordDef{
    pub const CSW_SIZE    :u32 = 0x00000007;  // Access Size: Selection Mask
    pub const CSW_SIZE8   :u32 = 0x00000000;  // Access Size: 8-bit
    pub const CSW_SIZE16  :u32 = 0x00000001;  // Access Size: 16-bit
    pub const CSW_SIZE32  :u32 = 0x00000002;  // Access Size: 32-bit
    pub const CSW_ADDRINC :u32 = 0x00000030;  // Auto Address Increment Mask
    pub const CSW_NADDRINC:u32 = 0x00000000;  // No Address Increment
    pub const CSW_SADDRINC:u32 = 0x00000010;  // Single Address Increment
    pub const CSW_PADDRINC:u32 = 0x00000020;  // Packed Address Increment
    pub const CSW_DBGSTAT :u32 = 0x00000040;  // Debug Status
    pub const CSW_TINPROG :u32 = 0x00000080;  // Transfer in progress
    pub const CSW_HPROT   :u32 = 0x02000000;  // User/Privilege Control
    pub const CSW_MSTRTYPE:u32 = 0x20000000;  // Master Type Mask
    pub const CSW_MSTRCORE:u32 = 0x00000000;  // Master Type: Core
    pub const CSW_MSTRDBG :u32 = 0x20000000;  // Master Type: Debug
    pub const CSW_RESERVED:u32 = 0x01000000;  // Reserved Value
}
// Core Debug Register Address Offsets
pub mod CoreDebugRegisterAdrOffsets{
    pub const DBG_OFS     :u32 = 0x0DF0;      // Debug Register Offset inside NVIC
    pub const DBG_HCSR_OFS:u32 = 0x00  ;      // Debug Halting Control & Status Register
    pub const DBG_CRSR_OFS:u32 = 0x04  ;      // Debug Core Register Selector Register
    pub const DBG_CRDR_OFS:u32 = 0x08  ;      // Debug Core Register Data Register
    pub const DBG_EMCR_OFS:u32 = 0x0C  ;      // Debug Exception & Monitor Control Register
}

// Core Debug Register Addresses
// #define DBG_HCSR       (DBG_Addr + DBG_HCSR_OFS)
// #define DBG_CRSR       (DBG_Addr + DBG_CRSR_OFS)
// #define DBG_CRDR       (DBG_Addr + DBG_CRDR_OFS)
// #define DBG_EMCR       (DBG_Addr + DBG_EMCR_OFS)

// // Debug Halting Control and Status Register definitions
// #define C_DEBUGEN      0x00000001  // Debug Enable
// #define C_HALT         0x00000002  // Halt
// #define C_STEP         0x00000004  // Step
// #define C_MASKINTS     0x00000008  // Mask Interrupts
// #define C_SNAPSTALL    0x00000020  // Snap Stall
// #define S_REGRDY       0x00010000  // Register R/W Ready Flag
// #define S_HALT         0x00020000  // Halt Flag
// #define S_SLEEP        0x00040000  // Sleep Flag
// #define S_LOCKUP       0x00080000  // Lockup Flag
// #define S_RETIRE_ST    0x01000000  // Sticky Retire Flag
// #define S_RESET_ST     0x02000000  // Sticky Reset Flag
// #define DBGKEY         0xA05F0000  // Debug Key

// // Debug Exception and Monitor Control Register definitions
// #define VC_CORERESET   0x00000001  // Reset Vector Catch
// #define VC_MMERR       0x00000010  // Debug Trap on MMU Fault
// #define VC_NOCPERR     0x00000020  // Debug Trap on No Coprocessor Fault
// #define VC_CHKERR      0x00000040  // Debug Trap on Checking Error Fault
// #define VC_STATERR     0x00000080  // Debug Trap on State Error Fault
// #define VC_BUSERR      0x00000100  // Debug Trap on Bus Error Fault
// #define VC_INTERR      0x00000200  // Debug Trap on Interrupt Error Fault
// #define VC_HARDERR     0x00000400  // Debug Trap on Hard Fault
// #define MON_EN         0x00010000  // Monitor Enable
// #define MON_PEND       0x00020000  // Monitor Pend
// #define MON_STEP       0x00040000  // Monitor Step
// #define MON_REQ        0x00080000  // Monitor Request
// #define TRCENA         0x01000000  // Trace Enable (DWT, ITM, ETM, TPIU)

// // NVIC: Interrupt Controller Type Register
// #define NVIC_ICT       (NVIC_Addr + 0x0004)
// #define INTLINESNUM    0x0000001F  // Interrupt Line Numbers

// // NVIC: CPUID Base Register
// #define NVIC_CPUID     (NVIC_Addr + 0x0D00)
// #define CPUID_PARTNO   0x0000FFF0  // Part Number Mask
// #define CPUID_REVISION 0x0000000F  // Revision Mask
// #define CPUID_VARIANT  0x00F00000  // Variant Mask

// // NVIC: Application Interrupt/Reset Control Register
// #define NVIC_AIRCR     (NVIC_Addr + 0x0D0C)
// #define VECTRESET      0x00000001  // Reset Cortex-M (except Debug)
// #define VECTCLRACTIVE  0x00000002  // Clear Active Vector Bit
// #define SYSRESETREQ    0x00000004  // Reset System (except Debug)
// #define VECTKEY        0x05FA0000  // Write Key

// // NVIC: Debug Fault Status Register
// #define NVIC_DFSR      (NVIC_Addr + 0x0D30)
// #define HALTED         0x00000001  // Halt Flag
// #define BKPT           0x00000002  // BKPT Flag
// #define DWTTRAP        0x00000004  // DWT Match
// #define VCATCH         0x00000008  // Vector Catch Flag
// #define EXTERNAL       0x00000010  // External Debug Request