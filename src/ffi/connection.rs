use std::ffi::{c_long, c_ushort};

use super::{provided_types::TVariant, string_utils::os_string_nil};

pub enum Error {
    None = 1000,
    Ordinary = 1001,
    Attention = 1002,
    Important = 1003,
    VeryImportant = 1004,
    Info = 1005,
    Fail = 1006,
    DialogAttention = 1007,
    DialogInfo = 1008,
    DialogFail = 1009,
}

#[repr(C)]
struct ConnectionVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    add_error: unsafe extern "system" fn(
        &Connection,
        c_ushort,
        *const u16,
        *const u16,
        c_long,
    ) -> bool,
    read: unsafe extern "system" fn(
        &Connection,
        *mut u16,
        &mut TVariant,
        c_long,
        *mut *mut u16,
    ) -> bool,
    write:
        unsafe extern "system" fn(&Connection, *mut u16, &mut TVariant) -> bool,
    register_profile_as:
        unsafe extern "system" fn(&Connection, *mut u16) -> bool,
    set_event_buffer_depth:
        unsafe extern "system" fn(&Connection, c_long) -> bool,
    get_event_buffer_depth: unsafe extern "system" fn(&Connection) -> c_long,
    external_event: unsafe extern "system" fn(
        &Connection,
        *mut u16,
        *mut u16,
        *mut u16,
    ) -> bool,
    clean_event_buffer: unsafe extern "system" fn(&Connection),
    set_status_line: unsafe extern "system" fn(&Connection, *mut u16) -> bool,
    reset_status_line: unsafe extern "system" fn(&Connection),
}

#[repr(C)]
pub struct Connection {
    vptr1: &'static ConnectionVTable,
}

impl Connection {
    pub fn add_error(
        &self,
        code: Error,
        source: &str,
        description: &str,
    ) -> bool {
        unsafe {
            let source_wstr = os_string_nil(source);
            let description_wstr = os_string_nil(description);
            (self.vptr1.add_error)(
                self,
                code as u16,
                source_wstr.as_ptr(),
                description_wstr.as_ptr(),
                0,
            )
        }
    }

    pub fn external_event(&self, caller: &str, name: &str, data: &str) -> bool {
        unsafe {
            let mut caller_wstr = os_string_nil(caller);
            let mut name_wstr = os_string_nil(name);
            let mut data_wstr = os_string_nil(data);
            (self.vptr1.external_event)(
                self,
                caller_wstr.as_mut_ptr(),
                name_wstr.as_mut_ptr(),
                data_wstr.as_mut_ptr(),
            )
        }
    }

    pub fn set_event_buffer_depth(&self, depth: c_long) -> bool {
        unsafe { (self.vptr1.set_event_buffer_depth)(self, depth) }
    }

    pub fn get_event_buffer_depth(&self) -> c_long {
        unsafe { (self.vptr1.get_event_buffer_depth)(self) }
    }
}
