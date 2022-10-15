//! Handles wifi networking using embedded-svc traits.

use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Status,
};

#[cfg(feature = "mcu-esp32c3")]
#[path = "esp.rs"]
mod ඞ;

use defmt::{debug, info};
use embassy_futures::yield_now;
use embedded_svc::wifi::Wifi;
use smoltcp::iface::SocketStorage;

const WIFI_SSID: &str = env!("FERROUS_SLIME_WIFI_SSID");
const WIFI_PWD: &str = env!("FERROUS_SLIME_WIFI_PASS");

pub async fn network_task(/* TODO: Figure out how to pass peripherals */) {
    debug!("Started network task");
    let ss = &mut [SocketStorage::EMPTY];
    let wifi = ඞ::create_wifi(ss /* TODO: pass peripherals */);
    debug!("Created wifi");
    let network = connect_to_wifi(wifi).await;
    info!("Connected to wifi");
    loop {
        //todo
        yield_now().await // Yield to ensure fairness
    }
}

async fn connect_to_wifi<W: Wifi>(mut wifi: W) -> Result<(), W::Error> {
    let client_cfg = ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PWD.into(),
        ..Default::default()
    };
    wifi.set_configuration(&embedded_svc::wifi::Configuration::Client(client_cfg))?;

    // TODO: Figure out how to handle never getting connected
    // wait to get connected
    let status = loop {
        if let Status(ClientStatus::Started(status), _) = wifi.get_status() {
            break status;
        }
    };
    debug!(
        "Started wifi connection: {:?}",
        defmt::Debug2Format(&status)
    );

    let status = loop {
        // wifi.poll_dhcp()?;
        if let Status(
            ClientStatus::Started(ClientConnectionStatus::Connected(
                ClientIpStatus::Done(status),
            )),
            _,
        ) = wifi.get_status()
        {
            break status;
        }
    };
    debug!("Connected to wifi: {:?}", defmt::Debug2Format(&status));

    Ok(())
}
