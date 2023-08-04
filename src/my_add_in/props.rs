use crate::{add_in::ComponentPropDescription, ffi::types::ParamValue};

use super::MyAddInDescription;

pub struct PropListElement {
    pub description: ComponentPropDescription,
    pub get_callback: Option<fn(&MyAddInDescription) -> Option<ParamValue>>,
    pub set_callback: Option<fn(&mut MyAddInDescription, &ParamValue) -> bool>,
}

impl MyAddInDescription {
    pub fn generate_prop_list() -> Vec<PropListElement> {
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
}
