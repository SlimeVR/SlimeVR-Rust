use embedded_svc::wifi::Wifi;
use smoltcp::iface::SocketStorage;

pub fn create_wifi<'a>(ss: &'a mut [SocketStorage<'a>]) -> impl Wifi + 'a {
    // TODO: `esp_wifi::initialize()` with peripherals.
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
