pub struct SwdConfig{
    pub clock_speed_mhz: u32,
    pub turnaround:u32 ,
    pub idle_cycles:u32,
    pub target_speed_khz: u32,
}
impl Default for SwdConfig{
    fn default() -> Self{
        Self { clock_speed_mhz: 160 ,turnaround: 1,idle_cycles:0,target_speed_khz: 4000}
    }
}