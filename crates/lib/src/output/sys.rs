use std::ffi::{c_int, c_void};
use windows_sys::Win32::Foundation::BOOL;

type LPStr = *const u8;

pub static OUTPUT_INFO_FLAG_VIDEO: c_int = 0x0001;
pub static OUTPUT_INFO_FLAG_AUDIO: c_int = 0x0002;
pub static OUTPUT_INFO_FLAG_BATCH: c_int = 0x0004;
pub static OUTPUT_INFO_FRAME_FLAG_KEYFRAME: c_int = 0x0001;
pub static OUTPUT_INFO_FRAME_FLAG_NONKEYFRAME: c_int = 0x0002;

#[derive(Debug)]
#[repr(C)]
pub struct OutputInfo {
    pub flag: c_int,
    pub w: c_int,
    pub h: c_int,
    pub rate: c_int,
    pub scale: c_int,
    pub n: c_int,
    pub size: c_int,
    pub audio_rate: c_int,
    pub audio_ch: c_int,
    pub audio_n: c_int,
    pub audio_size: c_int,
    pub savefile: LPStr,
    pub func_get_video: extern "C" fn(/* frame: */ c_int) -> *const u8,
    pub func_get_audio: extern "C" fn(
        /* start: */ c_int,
        /* length: */ c_int,
        /* readed: */ *mut c_int,
    ) -> *const u8,
    pub func_is_abort: extern "C" fn() -> BOOL,
    pub func_rest_time_disp: extern "C" fn(/* now: */ c_int, /* total: */ c_int) -> BOOL,
    pub func_get_flag: extern "C" fn(/* frame: */ c_int) -> c_int,
    pub func_update_preview: extern "C" fn() -> BOOL,
    pub func_get_video_ex:
        extern "C" fn(/* frame: */ c_int, /* format: */ [u8; 4]) -> *const u8,
}

#[repr(C)]
pub struct OutputPluginTable {
    pub flag: c_int,
    pub name: *const u8,
    pub filefilter: *const u8,
    pub information: *const u8,

    pub func_init: extern "C" fn() -> BOOL,
    pub func_exit: extern "C" fn() -> BOOL,
    pub func_output: extern "C" fn(/* oip: */ *const OutputInfo) -> BOOL,
    // ここはnullにもできるため、関数ポインタではなく*mut c_voidにしている。
    // pub func_config: extern "C" fn(/* hwnd: */ HWND, /* dll_hinst: */ HINSTANCE) -> BOOL,
    pub func_config: *mut c_void,
    pub func_config_get: extern "C" fn(/* data: */ *mut u8, /* size: */ c_int) -> c_int,
    pub func_config_set: extern "C" fn(/* data: */ *const u8, /* size: */ c_int) -> c_int,
}
