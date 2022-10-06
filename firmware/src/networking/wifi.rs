use defmt::{debug, info};
use embassy_futures::yield_now;
use embedded_svc::wifi::Wifi;
use smoltcp::iface::SocketStorage;

const WIFI_SSID: &str = env!("FERROUS_SLIME_WIFI_SSID");
const WIFI_PWD: &str = env!("FERROUS_SLIME_WIFI_PASS");

pub async fn network_task() {
    debug!("Started network task");
    let ss = &mut [SocketStorage::EMPTY];
    let wifi = create_wifi(ss);
    debug!("Created wifi");
    let network = connect_to_wifi(wifi).await;
    info!("Connected to wifi");
    loop {
        //todo
        yield_now().await // Yield to ensure fairness
    }
}

fn create_wifi<'a>(ss: &'a mut [SocketStorage<'a>]) -> impl Wifi + 'a {
    let device = esp_wifi::wifi::WifiDevice::new();
    let interface = {
        let builder = smoltcp::iface::InterfaceBuilder::new(
            device,
            managed::ManagedSlice::Borrowed(ss),
        );
        builder.finalize()
    };
    esp_wifi::wifi_interface::Wifi::new(interface)
}

async fn connect_to_wifi<W: Wifi>(mut wifi: W) {
    let (access_points, _) = wifi.scan_n().expect("Error while scanning");
    // TODO
}
