pub mod add_in;
mod ffi;
mod my_add_in;

use crate::ffi::create_component;
use add_in::{
    AddInContainer, ComponentFuncDescription, ComponentPropDescription,
};
use color_eyre::eyre::Result;
use ffi::{destroy_component, types::ParamValue, AttachType};
use my_add_in::AddInDescription;
use std::{
    ffi::{c_int, c_long, c_void},
    sync::atomic::{AtomicI32, Ordering},
};
use utf16_lit::utf16_null;

pub static mut PLATFORM_CAPABILITIES: AtomicI32 = AtomicI32::new(-1);

pub struct FunctionListElement {
    description: ComponentFuncDescription,
    callback:
        fn(&mut AddInDescription, &[ParamValue]) -> Result<Option<ParamValue>>,
}

pub struct PropListElement {
    description: ComponentPropDescription,
    get_callback: Option<fn(&AddInDescription) -> Option<ParamValue>>,
    set_callback: Option<fn(&mut AddInDescription, &ParamValue) -> bool>,
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn GetClassObject(
    name: *const u16,
    component: *mut *mut c_void,
) -> c_long {
    match *name as u8 {
        b'1' => {
            let my_add_in_container =
                AddInContainer::new(AddInDescription::new());
            create_component(component, my_add_in_container)
        }
        _ => 0,
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn DestroyObject(component: *mut *mut c_void) -> c_long {
    destroy_component(component)
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetClassNames() -> *const u16 {
    // small strings for performance
    utf16_null!("1").as_ptr()
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn SetPlatformCapabilities(capabilities: c_int) -> c_int {
    PLATFORM_CAPABILITIES.store(capabilities, Ordering::Relaxed);
    3
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetAttachType() -> AttachType {
    AttachType::Any
}
