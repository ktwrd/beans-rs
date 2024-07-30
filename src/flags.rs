use bitflags::bitflags;
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct LaunchFlag: u32
    {
        // debug mode, print full errors and other debug messages to console.
        const DEBUG_MODE = 0x01;
        // run from beans script
        const AUTOMATED = 0x02;
        // use the standard CLI wizard
        const WIZARD = 0x04;
        // enable experimental GUI for the wizard
        const WIZARD_GUI = 0x08;
        // please enable this flag when this is being used by a standalone application
        const STANDALONE_APP = 0x16;
    }
}
pub static mut LAUNCH_FLAGS: u32 = 0x00;

/// check if the `flag` provided is in `LAUNCH_FLAGS`
pub fn has_flag(flag: LaunchFlag) -> bool
{
    unsafe {
        let data = LaunchFlag::from_bits(LAUNCH_FLAGS).unwrap_or(LaunchFlag::empty());
        data.contains(flag)
    }
}

/// Add a flag to `LAUNCH_FLAGS`
pub fn add_flag(flag: LaunchFlag)
{
    unsafe {
        if let LaunchFlag::DEBUG_MODE = flag
        {
            crate::logger::LOG_FORMAT = crate::logger::LOG_FORMAT_DEFAULT;
        };

        let mut data = LaunchFlag::from_bits(LAUNCH_FLAGS).unwrap_or(LaunchFlag::empty());
        data.insert(flag);
        LAUNCH_FLAGS = data.bits();
    }
}

/// remove a flag from `LAUNCH_FLAGS`
pub fn remove_flag(flag: LaunchFlag)
{
    unsafe {
        if flag == LaunchFlag::DEBUG_MODE
        {
            crate::logger::LOG_FORMAT = crate::logger::LOG_FORMAT_MINIMAL;
        };
        let mut data = LaunchFlag::from_bits(LAUNCH_FLAGS).unwrap_or(LaunchFlag::empty());
        data.remove(flag);
        LAUNCH_FLAGS = data.bits();
    }
}

pub fn debug_mode() -> bool
{
    has_flag(LaunchFlag::DEBUG_MODE)
}
