use super::sys;
use super::OutputInfo;

use std::ffi::CString;
use std::sync::Mutex;

use encoding_rs::SHIFT_JIS;
use serde::de::DeserializeOwned;
use serde::Serialize;
use windows_sys::Win32::Foundation::BOOL;

pub use windows_sys::Win32::Foundation::{HINSTANCE, HWND};

/// 文字列をShift-JISにエンコードする。
fn encode(s: impl AsRef<str>) -> Vec<u8> {
    SHIFT_JIS.encode(s.as_ref()).0.to_vec()
}

#[derive(Debug)]
/// ファイルフィルタの情報。
pub struct FileFilterEntry {
    /// ファイルの名前。
    pub name: &'static str,
    /// ファイルの拡張子。
    pub filter: &'static str,
}

#[derive(Debug)]
/// 出力プラグインの情報。OUTPUT_PLUGIN_TABLEの関数以外の情報と対応。
pub struct OutputPluginTableInfo {
    /// プラグインの名前。
    pub name: &'static str,
    /// ファイルフィルタ。
    pub filefilter: &'static [FileFilterEntry],
    /// プラグインの情報。
    pub information: &'static str,
    /// コンフィグがあるかどうか。
    /// コンフィグがある場合、config関数が呼ばれる。
    pub has_config: bool,
}

/// 出力プラグインを登録するマクロ。
#[macro_export]
macro_rules! register_output_plugin {
    ($s:ident) => {
        static OUTPUT_PLUGIN: $s = $s();
        #[no_mangle]
        pub extern "stdcall" fn GetOutputPluginTable(
        ) -> *const aviutl_rs::output::sys::OutputPluginTable {
            aviutl_rs::output::_init(&OUTPUT_PLUGIN)
        }
    };
}

static TABLE: Mutex<Option<&dyn OutputPluginTable>> = Mutex::new(None);
pub fn _init<T: OutputPluginTable>(table: &'static T) -> *const sys::OutputPluginTable {
    if TABLE.lock().unwrap().is_some() {
        panic!("Output plugin already initialized");
    }
    let info = table.info();
    let name = encode(info.name);
    let mut filefilter: Vec<u8> = vec![];
    for entry in info.filefilter {
        filefilter.extend_from_slice(&encode(format!("{} ({})", entry.name, entry.filter)));
        filefilter.push(0);
        filefilter.extend_from_slice(&encode(entry.filter));
        filefilter.push(0);
    }
    filefilter.push(0);
    let information = encode(info.information);
    let name_ptr = CString::new(name).unwrap();
    let filefilter_box = filefilter.into_boxed_slice();
    let filefilter_ptr = Box::leak(filefilter_box).as_ptr();

    let information_ptr = CString::new(information).unwrap();
    TABLE.lock().unwrap().replace(table);
    let func_config = if info.has_config {
        config as _
    } else {
        std::ptr::null_mut()
    };
    let table_box = Box::new(sys::OutputPluginTable {
        flag: 0,
        name: name_ptr.into_raw() as _,
        filefilter: filefilter_ptr as _,
        information: information_ptr.into_raw() as _,
        func_init: init,
        func_exit: exit,
        func_output: output,
        func_config,
        func_config_get: config_get,
        func_config_set: config_set,
    });
    Box::leak(table_box) as *const sys::OutputPluginTable
}

/// 出力プラグイン。
#[allow(unused_variables)]
pub trait OutputPluginTable: Send + Sync {
    fn info(&self) -> OutputPluginTableInfo;
    fn init(&self) {}
    fn exit(&self) {}
    fn output(&self, info: &OutputInfo) -> anyhow::Result<()>;
    fn config(&self, hwnd: HWND, dll_hinst: HINSTANCE) {}
}

extern "C" fn init() -> BOOL {
    TABLE.lock().unwrap().unwrap().init();
    true.into()
}

extern "C" fn exit() -> BOOL {
    TABLE.lock().unwrap().unwrap().exit();
    true.into()
}

extern "C" fn output(oip: *const sys::OutputInfo) -> BOOL {
    let oip = unsafe { &*oip };
    let info = OutputInfo::from(oip);
    let result = TABLE.lock().unwrap().unwrap().output(&info);
    match result {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error(output): {}", e);
            false
        }
    }
    .into()
}

extern "C" fn config(hwnd: HWND, dll_hinst: HINSTANCE) -> BOOL {
    TABLE.lock().unwrap().unwrap().config(hwnd, dll_hinst);

    true.into()
}

static CONFIG: Mutex<Vec<u8>> = Mutex::new(vec![]);

extern "C" fn config_get(data: *mut u8, size: i32) -> i32 {
    let config = CONFIG.lock().unwrap();
    if data.is_null() {
        return config.len() as _;
    }
    assert!(size >= config.len() as _, "Config too large");
    unsafe { std::ptr::copy_nonoverlapping(config.as_ptr(), data, config.len()) };

    config.len() as _
}

extern "C" fn config_set(data: *const u8, size: i32) -> i32 {
    let mut config = CONFIG.lock().unwrap();
    config.clear();
    config.extend_from_slice(unsafe { std::slice::from_raw_parts(data, size as _) });

    size
}

pub fn get_config<T: DeserializeOwned>() -> Option<T> {
    let config_raw = CONFIG.lock().unwrap();
    if config_raw.is_empty() {
        None
    } else {
        Some(rmp_serde::decode::from_slice(&config_raw).unwrap())
    }
}

pub fn set_config<T: Serialize>(config: &T) {
    let mut config_raw = CONFIG.lock().unwrap();
    config_raw.clear();
    let new_config = rmp_serde::encode::to_vec(&config).unwrap();
    config_raw.extend(&new_config);
}
