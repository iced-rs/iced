use crate::handlers::SctkState;
use sctk::delegate_data_device_manager;
use std::fmt::Debug;

pub mod data_device;
pub mod data_offer;
pub mod data_source;

delegate_data_device_manager!(@<T: 'static + Debug> SctkState<T>);
