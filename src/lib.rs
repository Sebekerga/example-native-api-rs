pub mod add_in;
mod ffi;

use crate::ffi::create_component;
use add_in::{
    AddIn, AddInContainer, ComponentFuncDescription, ComponentPropDescription,
};
use ffi::{
    connection::Connection,
    destroy_component,
    types::ParamValue,
    utils::{from_os_string, os_string},
    AttachType,
};

use color_eyre::eyre::{eyre, Result};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use std::{
    ffi::{c_int, c_long, c_void},
    path::PathBuf,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use utf16_lit::utf16_null;

pub static mut PLATFORM_CAPABILITIES: AtomicI32 = AtomicI32::new(-1);

struct AddInDescription {
    name: &'static [u16],
    connection: Arc<Option<&'static Connection>>,
    log_handle: Option<log4rs::Handle>,

    functions: Vec<(
        ComponentFuncDescription,
        fn(&mut Self, &[ParamValue]) -> Result<Option<ParamValue>>,
    )>,

    some_prop_container: i32,
}

impl AddInDescription {
    pub fn new() -> Self {
        Self {
            name: &utf16_null!("MyAddIn"),
            connection: Arc::new(None),
            log_handle: None,
            functions: vec![
                (
                    ComponentFuncDescription::new::<0>(
                        vec![
                            &utf16_null!("Итерировать"),
                            &utf16_null!("Iterate"),
                        ],
                        false,
                        &[],
                    ),
                    Self::iterate,
                ),
                (
                    ComponentFuncDescription::new::<1>(
                        vec![&utf16_null!("Таймер"), &utf16_null!("Timer")],
                        true,
                        &[Some(ParamValue::I32(1000))],
                    ),
                    Self::timer,
                ),
                (
                    ComponentFuncDescription::new::<0>(
                        vec![
                            &utf16_null!("ПолучитьХэТэТэПэ"),
                            &utf16_null!("FetchHTTP"),
                        ],
                        true,
                        &[],
                    ),
                    Self::fetch,
                ),
                (
                    ComponentFuncDescription::new::<1>(
                        vec![
                            &utf16_null!("ИнициализироватьЛоггер"),
                            &utf16_null!("InitLogger"),
                        ],
                        false,
                        &[None],
                    ),
                    Self::init_logger,
                ),
            ],

            some_prop_container: 0,
        }
    }

    fn iterate(
        &mut self,
        _params: &[ParamValue],
    ) -> Result<Option<ParamValue>> {
        if self.some_prop_container >= 105 {
            return Err(eyre!("Prop is too big"));
        }
        self.some_prop_container += 1;
        log::info!("Prop is now {}", self.some_prop_container);
        Ok(None)
    }

    fn timer(&mut self, params: &[ParamValue]) -> Result<Option<ParamValue>> {
        let sleep_duration_ms = match params.get(0) {
            Some(ParamValue::I32(val)) => *val,
            _ => return Err(eyre!("Invalid parameter")),
        };
        if sleep_duration_ms < 0 {
            return Err(eyre!("Invalid parameter"));
        }
        if sleep_duration_ms > 100000 {
            return Err(eyre!("Too long"));
        }
        let sleep_duration_ms = sleep_duration_ms as u64;

        let connection = self.connection.clone();
        let name = from_os_string(self.name);
        thread::spawn(move || {
            log::info!("Timer started");
            thread::sleep(Duration::from_millis(sleep_duration_ms));
            log::info!("Timer ended");
            if let Some(connection) = &*connection {
                connection.external_event(&name, "TimerEnd", "OK");
            }
        });

        Ok(Some(ParamValue::I32(sleep_duration_ms as i32)))
    }

    fn fetch(&mut self, _params: &[ParamValue]) -> Result<Option<ParamValue>> {
        let Ok(result) = ureq::post("https://echo.hoppscotch.io").send_string("smth") else {return Err(eyre!("Failed to fetch"));};
        let Ok(body) = result.into_string() else { return Err(eyre!("Failed to get body"));};
        Ok(Some(ParamValue::Str(os_string(&body))))
    }

    fn init_logger(
        &mut self,
        params: &[ParamValue],
    ) -> Result<Option<ParamValue>> {
        let log_file_path = match params.get(0) {
            Some(ParamValue::Str(val)) => from_os_string(val),
            _ => return Err(eyre!("Invalid parameter")),
        };
        let log_file_path = PathBuf::from(log_file_path);
        if log_file_path.is_dir() {
            return Err(eyre!("Need a file path"));
        };

        let log_file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build(log_file_path)?;

        let config = Config::builder()
            .appender(
                Appender::builder().build("file", Box::new(log_file_appender)),
            )
            .build(Root::builder().appender("file").build(LevelFilter::Info))?;

        self.log_handle = Some(log4rs::init_config(config)?);
        log::info!("Logger initialized");
        Ok(None)
    }
}

impl AddIn for AddInDescription {
    fn init(&mut self, interface: &'static Connection) -> bool {
        interface.set_event_buffer_depth(10);
        self.connection = Arc::new(Some(interface));
        self.some_prop_container = 100;
        true
    }

    fn add_in_name(&self) -> &'static [u16] {
        self.name
    }

    fn call_function(
        &mut self,
        name: &str,
        params: &[ParamValue],
    ) -> Result<Option<ParamValue>> {
        let func = self
            .functions
            .iter()
            .find(|(desc, _)| desc.str_names().iter().any(|n| n == name));

        let Some(func) = func.map(|(_, callback)| callback) else { return Err(eyre!("No function with such name")) };
        func(self, params)
    }

    fn get_parameter(&self, name: &str) -> Option<ParamValue> {
        match name {
            "prop" => Some(ParamValue::I32(self.some_prop_container)),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: &ParamValue) -> bool {
        match name {
            "prop" => {
                let ParamValue::I32(val) = value else { return false };
                self.some_prop_container = *val;
                true
            }
            _ => false,
        }
    }

    fn list_functions(&self) -> Vec<&ComponentFuncDescription> {
        self.functions.iter().map(|(desc, _)| desc).collect()
    }

    fn list_parameters(&self) -> Vec<ComponentPropDescription> {
        vec![ComponentPropDescription {
            name: &utf16_null!("prop"),
            readable: true,
            writable: true,
        }]
    }
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
