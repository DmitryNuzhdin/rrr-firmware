use embedded_svc::wifi::{Configuration, AccessPointConfiguration, ClientConfiguration};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::info;
use anyhow::Result;
use esp_idf_sys::EspError;
use crate::api;


pub struct WiFi {
    configuration: api::WifiConnectionConfiguration,
    wifi: BlockingWifi<EspWifi<'static>>,
}

pub enum WiFiResult {
    Client,
    AP,
}

impl WiFi {

    pub fn new(
        configuration: api::WifiConnectionConfiguration,
        modem: impl Peripheral<P=esp_idf_hal::modem::Modem> + 'static,
        sysloop: EspSystemEventLoop,
    ) -> Result<Self> {
        let esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
        let mut wifi = BlockingWifi::wrap(esp_wifi, sysloop)?;

        let mut result: WiFiResult = WiFiResult::Client;

        wifi.set_configuration(&Configuration::AccessPoint(
            AccessPointConfiguration {
                ssid: "RRR-wifi".into(),
                channel: 1,
                ..Default::default()
            },
        ))?;

        let client_configuration_result = wifi.set_configuration(&Configuration::Client(
            ClientConfiguration {
                ssid: heapless::String::from(configuration.ssid.as_str()),
                password: heapless::String::from(configuration.password.as_str()),
                channel: None,
                ..Default::default()
            },
        ));

        let connection_result = client_configuration_result.and_then(|_| {
            wifi.start()?;
            wifi.connect()?;
            info!("WIFI Connect -- OK");
            Ok(())
        });

        match connection_result {
            Ok(_) => (),
            Err(_) => {
                info!("WIFI Connect -- FAIL");
                let ap_configuration_result = wifi.set_configuration(&Configuration::AccessPoint(
                    AccessPointConfiguration {
                        ssid: "RRR-wifi".into(),
                        channel: 1,
                        ..Default::default()
                    },
                ));
                ap_configuration_result.and_then(|_| {
                    wifi.start()?;
                    info!("WIFI AP Start -- OK");
                    result = WiFiResult::AP;
                    Ok(())
                })?;
            }
        };

        wifi.wait_netif_up()?;
        let ip_info = wifi.wifi().ap_netif().get_ip_info()?;
        info!("DHCP -- OK");
        info!("DHCP info: {:?}", ip_info);
        Ok(Self { configuration, wifi })
    }
}