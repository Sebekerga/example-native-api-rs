use super::MyAddInDescription;
use crate::add_in::ComponentFuncDescription;
use crate::ffi::utils::os_string;
use crate::ffi::{types::ParamValue, utils::from_os_string};
use color_eyre::eyre::{eyre, Result};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use std::{path::PathBuf, thread, time::Duration};

pub struct FunctionListElement {
    pub description: ComponentFuncDescription,
    pub callback: fn(
        &mut MyAddInDescription,
        &[ParamValue],
    ) -> Result<Option<ParamValue>>,
}

impl MyAddInDescription {
    pub fn generate_func_list() -> Vec<FunctionListElement> {
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
