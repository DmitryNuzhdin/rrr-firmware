use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct State {
    pub battery: BatteryState,
    pub pyro: PyroState,
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BatteryState {
    pub soc: f32,
    pub voltage: f32,
    pub charge_rate: f32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WifiConnectionConfiguration {
    pub connection_type: WifiConnectionType,
    pub ssid: String,
    pub password: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum WifiConnectionType {
    ConnectToExternal,
    StartAccessPoint,
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PyroChannelState {
    pub fire: bool,
    pub test_voltage: f32,
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PyroState {
    pub channel1: PyroChannelState,
    pub channel2: PyroChannelState,
}


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Reset,
    SetWifi { ssid: String, password: String },
    SetLedColor { r: u8, g: u8, b: u8 },
    SetPwmDutyCycle { duty_cycle: f32 },
}