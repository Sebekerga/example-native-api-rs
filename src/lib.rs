pub mod add_in;
mod export_functions;
mod ffi;
mod my_add_in;

use add_in::AddIn;
use my_add_in::AddInDescription;

pub fn init_my_add_in() -> impl AddIn {
    AddInDescription::new()
}
