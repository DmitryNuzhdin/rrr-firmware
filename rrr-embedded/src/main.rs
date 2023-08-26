mod led_driver;

use crate::led_driver::LedDriver;

use std::sync::{Arc, Mutex};
use log::*;
use anyhow::Result;
use embedded_svc::wifi::*;

use esp_idf_svc::eventloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::peripheral;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::*;
use esp_idf_hal::delay::FreeRtos;

use max170xx::Max17048;


const SSID: &str = include_str!("../wifi-ssid.secret");
const PASS: &str = include_str!("../wifi-password.secret");
const INDEX: &[u8] = include_bytes!("../../rrr-frontend/dist/index.html.gz");
const CSS: &[u8] = include_bytes!("../../rrr-frontend/dist/index.css.gz");
const WASM: &[u8] = include_bytes!("../../rrr-frontend/dist/rrr-frontend_bg.wasm.gz");
const JS: &[u8] = include_bytes!("../../rrr-frontend/dist/rrr-frontend.js.gz");

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let i2c = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0.into_ref(),
        peripherals.pins.gpio7,
        peripherals.pins.gpio8,
        &esp_idf_hal::i2c::I2cConfig::default(),
    )?;

    let mut max17048 = Max17048::new(i2c);

    max17048.version().unwrap();
    info!("SOC: {:.2}", max17048.soc().unwrap());
    info!("MAX -- OK");


    //Drivers
    let mut led_driver = LedDriver::new(9, 0)?;
    info!("LED -- OK");
    led_driver.set_rgb(20, 0, 0)?;


    #[allow(unused_variables)]
        let wifi = wifi(peripherals.modem, sysloop.clone())?;
    info!("WIFI -- OK");
    led_driver.set_rgb(0, 20, 0)?;

    #[allow(unused_variables)]
        let httpd = httpd(Arc::new(Mutex::new(led_driver)), Arc::new(Mutex::new(max17048)))?;

    info!("HTTP server -- OK");

    let mut mdns = esp_idf_svc::mdns::EspMdns::take()?;
    mdns.set_hostname("rrr")?;
    mdns.set_instance_name("RRR web server")?;
    mdns.add_service(None, "_http", "_tcp", 80, &[("board", "{esp32}")])?;

    info!("mDNS -- OK");

    loop {
        FreeRtos::delay_ms(1000);
    }

    Ok(())
}

fn httpd(led: Arc<Mutex<LedDriver>>, max17048: Arc<Mutex<Max17048<I2cDriver<'static>>>>) -> Result<esp_idf_svc::http::server::EspHttpServer> {
    use embedded_svc::http::server::{Method};
    use embedded_svc::io::Write;
    use esp_idf_svc::http::server::{EspHttpServer};

    let mut server = EspHttpServer::new(&Default::default())?;

    let l1 = led.clone();
    let l2 = led.clone();
    let l3 = led.clone();
    let l4 = led.clone();

    server
        .fn_handler("/batt", Method::Get, move |req| {
            req.into_ok_response()?
                .write_all({
                    let mut m = max17048.lock().unwrap();
                    format!(
                        "Battery charge: {:.2}%, voltage: {:.2}, discharge rate: {:.2}",
                        m.soc().unwrap(),
                        m.voltage().unwrap(),
                        m.charge_rate().unwrap()
                    ).as_bytes()
                })?;
            Ok(())
        })?
        .fn_handler("/red", Method::Get, move |req| {
            req.into_ok_response()?.write_all("RED".as_bytes())?;
            l1.lock().unwrap().set_rgb(20, 0, 0)?;
            Ok(())
        })?
        .fn_handler("/green", Method::Get, move |req| {
            req.into_ok_response()?.write_all("GREEN".as_bytes())?;
            l2.lock().unwrap().set_rgb(0, 20, 0)?;
            Ok(())
        })?
        .fn_handler("/blue", Method::Get, move |req| {
            req.into_response(200, None, &[("Content-Type", "image/jpg"),
                ("Content-Encoding", "gzip"),
                ("Access-Control-Allow-Origin", "*"),
            ])?.write_all("BLUE".as_bytes())?;
            l3.lock().unwrap().set_rgb(0, 0, 20)?;
            Ok(())
        })?
        .fn_handler("/off", Method::Get, move |req| {
            req.into_ok_response()?.write_all("OFF".as_bytes())?;
            l4.lock().unwrap().off()?;
            Ok(())
        })?
        .fn_handler("/", Method::Get, move |req| {
            req.into_response(200, None, &[("Content-Type", "text/html"),
                ("Content-Encoding", "gzip"),
                ("Access-Control-Allow-Origin", "*"),
            ])?.write_all(INDEX)?;
            Ok(())
        })?
        .fn_handler("/index.css", Method::Get, move |req| {
            req.into_response(200, None, &[("Content-Type", "text/css"),
                ("Content-Encoding", "gzip"),
                ("Access-Control-Allow-Origin", "*"),
            ])?.write_all(CSS)?;
            Ok(())
        })?
        .fn_handler("/rrr-frontend_bg.wasm", Method::Get, move |req| {
            req.into_response(200, None, &[("Content-Type", "application/wasm"),
                ("Content-Encoding", "gzip"),
                ("Access-Control-Allow-Origin", "*"),
            ])?.write_all(WASM)?;
            Ok(())
        })?
        .fn_handler("/rrr-frontend.js", Method::Get, move |req| {
            req.into_response(200, None, &[("Content-Type", "application/javascript"),
                ("Content-Encoding", "gzip"),
                ("Access-Control-Allow-Origin", "*"),
            ])?.write_all(JS)?;
            Ok(())
        })?
    ;

    Ok(server)
}

fn wifi(
    modem: impl peripheral::Peripheral<P=esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::AccessPoint(
        AccessPointConfiguration {
            ssid: "RRR-wifi".into(),
            channel: 1,
            ..Default::default()
        },
    ))?;

    wifi.set_configuration(&Configuration::Client(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel: None,
            ..Default::default()
        },
    ))?;

    wifi.start()?;
    info!("WIFI Start -- OK");
    wifi.connect()?;
    info!("WIFI Connect -- OK");
    wifi.wait_netif_up()?;
    let ip_info = wifi.wifi().ap_netif().get_ip_info()?;
    info!("DHCP -- OK");
    info!("DHCP info: {:?}", ip_info);
    Ok(Box::new(esp_wifi))
}