//#[link(name = "

use esp_idf_hal::sys::{__int32_t, adc_channel_t, adc_continuous_handle_t, adc_unit_t, esp_err_t};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct adc_monitor_t;
pub type adc_monitor_handle_t = *mut adc_monitor_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct adc_monitor_config_t {
    pub adc_unit: adc_unit_t,
    pub channel: adc_channel_t,
    pub h_threshold: i32,
    pub l_threshold: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct adc_monitor_evt_data_t {}

pub type adc_monitor_evt_cb_t =
    fn(adc_monitor_handle_t, *const adc_monitor_evt_data_t, *const ::core::ffi::c_void) -> bool;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct adc_monitor_evt_cbs_t {
    pub on_over_high_thresh: adc_monitor_evt_cb_t,
    pub on_below_low_thresh: adc_monitor_evt_cb_t,
}

//#[link(name = " 
extern "C" {
    pub fn adc_new_continuous_monitor(
        handle: adc_continuous_handle_t,
        monitor_cfg: *const adc_monitor_config_t,
        ret_handle: *const adc_monitor_handle_t,
    ) -> esp_err_t;

    pub fn adc_continuous_monitor_register_event_callbacks(
        monitor_handle: adc_monitor_handle_t,
        cbs: *const adc_monitor_evt_cbs_t,
        userdata: *const ::core::ffi::c_void,
    ) -> esp_err_t;

    pub fn adc_continuous_monitor_enable(monitor_handle: adc_monitor_handle_t) -> esp_err_t;

    pub fn adc_continuous_monitor_disable(monitor_handle: adc_monitor_handle_t) -> esp_err_t;

    pub fn adc_del_continuous_monitor(monitor_handle: adc_monitor_handle_t) -> esp_err_t;

}
