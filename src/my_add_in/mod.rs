mod func;
mod props;

use crate::add_in::{
    AddIn, ComponentFuncDescription, ComponentPropDescription,
};
use crate::ffi::{connection::Connection, types::ParamValue};
use color_eyre::eyre::{eyre, Result};
use std::sync::Arc;
use utf16_lit::utf16_null;

use self::func::FunctionListElement;
use self::props::PropListElement;

pub struct MyAddInDescription {
    name: &'static [u16],
    connection: Arc<Option<&'static Connection>>,
    log_handle: Option<log4rs::Handle>,

    functions: Vec<FunctionListElement>,
    props: Vec<PropListElement>,
    some_prop_container: i32,
}

impl MyAddInDescription {
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
}

impl AddIn for MyAddInDescription {
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
