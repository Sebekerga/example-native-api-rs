use crate::add_in::{
    AddIn, ComponentFuncDescription, ComponentPropDescription,
};
use crate::ffi::{
    connection::Connection,
    types::ParamValue,
    utils::{from_os_string, os_string},
};
use crate::{FunctionListElement, PropListElement};

use color_eyre::eyre::{eyre, Result};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use std::{path::PathBuf, sync::Arc, thread, time::Duration};
use utf16_lit::utf16_null;

pub struct AddInDescription {
    name: &'static [u16],
    connection: Arc<Option<&'static Connection>>,
    log_handle: Option<log4rs::Handle>,

    functions: Vec<FunctionListElement>,
    props: Vec<PropListElement>,
    some_prop_container: i32,
}

impl AddInDescription {
    pub fn new() -> Self {
        Self {
            name: &utf16_null!("MyAddIn"),
            connection: Arc::new(None),
            log_handle: None,
            functions: Self::generate_func_list(),
            props: Self::generate_prop_list(),

            some_prop_container: 0,
        }
    }

    fn generate_func_list() -> Vec<FunctionListElement> {
        vec![
            FunctionListElement {
                description: ComponentFuncDescription::new::<0>(
                    &["Итерировать", "Iterate"],
                    false,
                    &[],
                ),
                callback: Self::iterate,
            },
            FunctionListElement {
                description: ComponentFuncDescription::new::<1>(
                    &["Таймер", "Timer"],
                    true,
                    &[Some(ParamValue::I32(1000))],
                ),
                callback: Self::timer,
            },
            FunctionListElement {
                description: ComponentFuncDescription::new::<0>(
                    &["ПолучитьХэТэТэПэ", "FetchHTTP"],
                    true,
                    &[],
                ),
                callback: Self::fetch,
            },
            FunctionListElement {
                description: ComponentFuncDescription::new::<1>(
                    &[("ИнициализироватьЛоггер"), ("InitLogger")],
                    false,
                    &[None],
                ),
                callback: Self::init_logger,
            },
        ]
    }

    fn generate_prop_list() -> Vec<PropListElement> {
        vec![PropListElement {
            description: ComponentPropDescription {
                names: &["prop"],
                readable: true,
                writable: true,
            },
            get_callback: Some(Self::get_prop),
            set_callback: Some(Self::set_prop),
        }]
    }

    fn get_prop(&self) -> Option<ParamValue> {
        Some(ParamValue::I32(self.some_prop_container))
    }

    fn set_prop(&mut self, value: &ParamValue) -> bool {
        match value {
            ParamValue::I32(val) => {
                self.some_prop_container = *val;
                true
            }
            _ => false,
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
            .find(|el| el.description.names.iter().any(|n| n == &name));

        let Some(func) = func.map(|el| el.callback) else { return Err(eyre!("No function with such name")) };
        func(self, params)
    }

    fn get_parameter(&self, name: &str) -> Option<ParamValue> {
        let prop = self
            .props
            .iter()
            .find(|el| el.description.names.iter().any(|n| n == &name));
        let Some(Some(get)) = prop.map(|el| el.get_callback) else { return None };
        get(self)
    }

    fn set_parameter(&mut self, name: &str, value: &ParamValue) -> bool {
        let prop = self
            .props
            .iter()
            .find(|el| el.description.names.iter().any(|n| n == &name));
        let Some(Some(set)) = prop.map(|el| el.set_callback) else { return false };
        set(self, value)
    }

    fn list_functions(&self) -> Vec<&ComponentFuncDescription> {
        self.functions.iter().map(|el| &el.description).collect()
    }

    fn list_parameters(&self) -> Vec<&ComponentPropDescription> {
        self.props.iter().map(|el| &el.description).collect()
    }
}
