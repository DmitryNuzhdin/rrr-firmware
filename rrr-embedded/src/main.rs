mod led_driver;
mod ota;
mod wifi;
mod server;
mod nvs;

use crate::led_driver::LedDriver;
use crate::ota::OtaDriver;

use rrr_api as api;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use log::*;
use anyhow::Result;
use embedded_svc::http::server::Connection;
use embedded_svc::io::Read;
use embedded_svc::wifi::*;
use esp_idf_hal::adc::{ADC1, AdcChannelDriver, Atten11dB};
use esp_idf_hal::adc::config::Resolution;

use esp_idf_svc::eventloop::*;
use esp_idf_svc::wifi::*;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::{Gpio1};
use esp_idf_hal::ledc;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::ledc::config::TimerConfig;
use max170xx::Max17048;
use rrr_api::WifiCredentials;
use crate::api::{Command, WifiConnectionConfiguration, WifiConnectionType};
use crate::server::Server;
use crate::wifi::WiFi;

static SSID: &str = include_str!("../wifi-ssid.secret");
static PASS: &str = include_str!("../wifi-password.secret");

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let state = Arc::new(Mutex::new(api::State::default()));

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let i2c = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0.into_ref(),
        peripherals.pins.gpio7,
        peripherals.pins.gpio8,
        &esp_idf_hal::i2c::I2cConfig::default(),
    )?;

    let _pyro = esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio6)?;


    let timer_driver =
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &TimerConfig::default()
                .frequency(50.Hz().into())
                .resolution(ledc::Resolution::Bits14),
        )?;
    let mut pwm_driver = LedcDriver::new(peripherals.ledc.channel0, timer_driver, peripherals.pins.gpio4)?;

    let max_duty = pwm_driver.get_max_duty();
    pwm_driver.set_duty(max_duty * 15 / 200)?;


    let mut max17048 = Max17048::new(i2c);

    max17048.version().unwrap();
    info!("SOC: {:.2}", max17048.soc().unwrap());
    info!("MAX -- OK");

    let max17048 = Arc::new(Mutex::new(max17048));

    esp_idf_hal::task::thread::ThreadSpawnConfiguration {
        name: Some(b"max-thread\0"),
        ..Default::default()
    }.set().unwrap();

    let max1 = max17048.clone();
    let max2 = max17048.clone();

    let state1 = state.clone();
    let state2 = state.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(1000));
            let mut state = state1.lock().unwrap();
            let mut max = max1.lock().unwrap();
            state.battery.soc = max.soc().unwrap();
            state.battery.voltage = max.voltage().unwrap();
            state.battery.charge_rate = max.charge_rate().unwrap();
        }
    });

    let mut adc_driver_config = esp_idf_hal::adc::AdcConfig::default();
    adc_driver_config.resolution = Resolution::Resolution12Bit;
    adc_driver_config.calibration = true;

    let mut adc_driver = esp_idf_hal::adc::AdcDriver::new(peripherals.adc1, &esp_idf_hal::adc::AdcConfig::default())?;
    let mut adc_channel_driver: AdcChannelDriver<'_, Gpio1, Atten11dB<ADC1>> = esp_idf_hal::adc::AdcChannelDriver::new(peripherals.pins.gpio1)?;

    adc_driver.read(&mut adc_channel_driver)?;

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(1000));
            let adc_reading = adc_driver.read(&mut adc_channel_driver).unwrap();
            let voltage = adc_reading as f32;
            let mut state = state2.lock().unwrap();
            state.pyro.channel1.test_voltage = voltage / 1000f32;
        }
    });


    //Drivers
    let mut led_driver = LedDriver::new(9, 0)?;
    info!("LED -- OK");
    led_driver.set_rgb(20, 0, 0)?;

    let mut nvs = nvs::Nvs::new()?;

    let nvs_arc0 = Arc::new(Mutex::new(nvs));
    let nvs_arc1 = nvs_arc0.clone();


    let wifi_configuration = match nvs_arc0.lock().unwrap().get_wifi_connection()? {
        None => {
            WifiConnectionConfiguration {
                connection_type: WifiConnectionType::StartAccessPoint,
                credentials: WifiCredentials {
                    ssid: String::from("RRR-wifi-0"),
                    password: String::from("12345678"),
                },
            }
        }
        Some(creds) => {
            WifiConnectionConfiguration {
                connection_type: WifiConnectionType::ConnectToExternal,
                credentials: WifiCredentials {
                    ssid: creds.ssid,
                    password: creds.password,
                },
            }
        }
    };

    #[allow(unused_variables)]
        let wifi = WiFi::new(wifi_configuration, peripherals.modem, sysloop.clone(), state.clone())?;

    match state.lock().unwrap().wifi_state.connection_type {
         WifiConnectionType::ConnectToExternal => led_driver.set_rgb(0, 20, 0)?,
        _ => led_driver.set_rgb(10, 10, 0)?,
    }

    #[allow(unused_variables)]
        let mut ota_driver = OtaDriver::new()?;


    let ld = Arc::new(Mutex::new(led_driver));

    let command_handler = move |c: &Command| -> Result<()> {
        match c {
            Command::Reset => {}
            Command::SetWifi { ssid, password } => {
                let creds = WifiCredentials { ssid: ssid.clone(), password: password.clone() };
                nvs_arc1.lock().unwrap().set_wifi_connection(creds).unwrap();
                nvs_arc1.lock().unwrap().get_wifi_connection().unwrap();
            }
            Command::SetLedColor { r, g, b } =>
                { ld.lock().unwrap().set_rgb(r.clone(), g.clone(), b.clone())? }
            _ => {}
        }
        Ok(())
    };

    #[allow(unused_variables)]
        let server = Server::new(state, command_handler)?;

    info!("HTTP server -- OK");
    info!("mDNS -- OK");

    let mut mdns = esp_idf_svc::mdns::EspMdns::take()?;
    mdns.set_hostname("rrr")?;
    mdns.set_instance_name("RRR web server")?;
    mdns.add_service(None, "_http", "_tcp", 80, &[("board", "{esp32}")])?;

    loop {
        thread::sleep(Duration::from_millis(1000));
    }

    #[allow(unreachable_code)]
    Ok(())
}