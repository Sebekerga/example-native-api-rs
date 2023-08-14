use std::{
    ffi::c_int,
    ptr,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use chrono::{Datelike, Offset, Timelike};

use super::memory_manager::MemoryManager;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Tm {
    pub sec: c_int, // seconds after the minute - [0, 60] including leap second
    pub min: c_int, // minutes after the hour - [0, 59]
    pub hour: c_int, // hours since midnight - [0, 23]
    pub mday: c_int, // day of the month - [1, 31]
    pub mon: c_int, // months since January - [0, 11]
    pub year: c_int, // years since 1900
    pub wday: c_int, // days since Sunday - [0, 6]
    pub yday: c_int, // days since January 1 - [0, 365]
    pub isdst: c_int, // daylight savings time flag

    #[cfg(target_family = "unix")]
    pub gmtoff: std::ffi::c_long, // seconds east of UTC
    #[cfg(target_family = "unix")]
    pub zone: std::ffi::c_char, // timezone abbreviation
}

impl From<chrono::DateTime<chrono::FixedOffset>> for Tm {
    fn from(dt: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self {
            sec: dt.second() as c_int,
            min: dt.minute() as c_int,
            hour: dt.hour() as c_int,
            mday: dt.day() as c_int,
            mon: dt.month() as c_int,
            year: dt.year() as c_int,
            wday: dt.weekday().num_days_from_sunday() as c_int,
            yday: dt.ordinal() as c_int,
            isdst: dt.timestamp() as c_int,
            gmtoff: dt.offset().fix().local_minus_utc() as std::ffi::c_long,
            zone: dt.offset().to_string().into_bytes()[0] as std::ffi::c_char,
        }
    }
}

impl From<Tm> for chrono::DateTime<chrono::FixedOffset> {
    fn from(tm: Tm) -> Self {
        let Some(naive_date) = chrono::NaiveDate::from_ymd_opt(
            tm.year,
            tm.mon as u32,
            tm.mday as u32,
        ) else {
            return chrono::DateTime::default();
        };
        let Some(naive_time) = chrono::NaiveTime::from_hms_opt(
            tm.hour as u32,
            tm.min as u32,
            tm.sec as u32,
        ) else {
            return chrono::DateTime::default();
        };
        let Some(offset) = chrono::FixedOffset::east_opt(tm.gmtoff as i32) else {
            return chrono::DateTime::default();
        };
        chrono::DateTime::from_utc(
            chrono::NaiveDateTime::new(naive_date, naive_time),
            offset,
        )
    }
}

pub struct ReturnValue<'a> {
    pub mem: &'a MemoryManager,
    pub variant: &'a mut TVariant,
    pub result: &'a mut bool,
}

#[allow(dead_code)]
impl<'a> ReturnValue<'a> {
    pub fn set_empty(self) {
        self.variant.vt = VariantType::Empty;
    }
    pub fn set_i32(self, val: i32) {
        self.variant.vt = VariantType::Int32;
        self.variant.value.i32 = val;
    }

    pub fn set_bool(self, val: bool) {
        self.variant.vt = VariantType::Bool;
        self.variant.value.bool = val;
    }

    pub fn set_f64(self, val: f64) {
        self.variant.vt = VariantType::Double;
        self.variant.value.f64 = val;
    }

    pub fn set_date(self, val: Tm) {
        self.variant.vt = VariantType::Time;
        self.variant.value.tm = val;
    }

    pub fn set_str(self, val: &[u16]) {
        let Some(ptr) = self.mem.alloc_str(val.len()) else {
            *self.result = false;
            return;
        };

        unsafe {
            ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr(), val.len())
        };

        self.variant.vt = VariantType::WStr;
        self.variant.value.data_str.ptr = ptr.as_ptr();
        self.variant.value.data_str.len = val.len() as u32;
    }

    pub fn set_blob(self, val: &[u8]) {
        let Some(ptr) = self.mem.alloc_blob(val.len()) else {
            *self.result = false;
            return;
        };

        unsafe {
            ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr(), val.len())
        };

        self.variant.vt = VariantType::Blob;
        self.variant.value.data_blob.ptr = ptr.as_ptr();
        self.variant.value.data_blob.len = val.len() as u32;
    }

    pub fn alloc_str(self, len: usize) -> Option<&'a mut [u16]> {
        let Some(ptr) = self.mem.alloc_str(len) else {
            *self.result = false;
            return None;
        };

        unsafe { ptr::write_bytes(ptr.as_ptr(), 0, len) };

        self.variant.vt = VariantType::WStr;
        self.variant.value.data_str.ptr = ptr.as_ptr();
        self.variant.value.data_str.len = len as u32;

        Some(unsafe { from_raw_parts_mut(ptr.as_ptr(), len) })
    }

    pub fn alloc_blob(self, len: usize) -> Option<&'a mut [u8]> {
        let Some(ptr) = self.mem.alloc_blob(len) else {
            *self.result = false;
            return None;
        };

        unsafe { ptr::write_bytes(ptr.as_ptr(), 0, len) };

        self.variant.vt = VariantType::Blob;
        self.variant.value.data_blob.ptr = ptr.as_ptr();
        self.variant.value.data_blob.len = len as u32;

        Some(unsafe { from_raw_parts_mut(ptr.as_ptr(), len) })
    }
}

pub enum ParamValue {
    Empty,
    Bool(bool),
    I32(i32),
    F64(f64),
    Date(Tm),
    Str(Vec<u16>),
    Blob(Vec<u8>),
}

impl<'a> From<&'a TVariant> for ParamValue {
    fn from(param: &'a TVariant) -> ParamValue {
        unsafe {
            match param.vt {
                VariantType::Empty => Self::Empty,
                VariantType::Bool => Self::Bool(param.value.bool),
                VariantType::Int32 => Self::I32(param.value.i32),
                VariantType::Double => Self::F64(param.value.f64),
                VariantType::Time => Self::Date(param.value.tm),
                VariantType::WStr => Self::Str(
                    from_raw_parts(
                        param.value.data_str.ptr,
                        param.value.data_str.len as usize,
                    )
                    .into(),
                ),
                VariantType::Blob => Self::Blob(
                    from_raw_parts(
                        param.value.data_blob.ptr,
                        param.value.data_blob.len as usize,
                    )
                    .into(),
                ),
                _ => Self::Empty,
            }
        }
    }
}

#[repr(u16)]
#[allow(dead_code)]
enum VariantType {
    Empty = 0,
    Null,
    Int16,     //int16_t
    Int32,     //int32_t
    Float,     //float
    Double,    //double
    Date,      //DATE (double)
    Time,      //struct tm
    PStr,      //struct str    string
    Interface, //struct iface
    Error,     //int32_t errCode
    Bool,      //bool
    Variant,   //struct _tVariant *
    Int8,      //int8_t
    UInt8,     //uint8_t
    UInt16,    //uint16_t
    UInt32,    //uint32_t
    Int64,     //int64_t
    UInt64,    //uint64_t
    Int,       //int   Depends on architecture
    UInt,      //unsigned int  Depends on architecture
    HResult,   //long hRes
    WStr,      //struct wstr
    Blob,      //means in struct str binary data contain
    ClsID,     //UUID

    Undefined = 0xFFFF,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DataStr {
    pub ptr: *mut u16,
    pub len: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DataBlob {
    pub ptr: *mut u8,
    pub len: u32,
}

#[repr(C)]
union VariantValue {
    pub bool: bool,
    pub i32: i32,
    pub f64: f64,
    pub tm: Tm,
    pub data_str: DataStr,
    pub data_blob: DataBlob,
}

#[repr(C)]
pub struct TVariant {
    value: VariantValue,
    elements: u32, //Dimension for an one-dimensional array in pvarVal
    vt: VariantType,
}
