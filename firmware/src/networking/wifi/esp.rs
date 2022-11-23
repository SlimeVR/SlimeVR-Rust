use defmt::{debug, error, info, trace};
use embassy_executor::task;
use embassy_futures::yield_now;
use embedded_svc::ipv4::Interface;
use esp_wifi::{
	create_network_stack_storage, current_millis, network_stack_storage,
	wifi::utils::create_network_interface, wifi_interface::Network,
};

#[task]
pub async fn network_task() {
	let mut storage = create_network_stack_storage!(3, 8, 1, 1);
	let ethernet = create_network_interface(network_stack_storage!(storage));
	let mut wifi = esp_wifi::wifi_interface::Wifi::new(ethernet);
	super::connect_wifi(&mut wifi)
		.await
		.expect("Couldn't connect to wifi");

	// wait for getting an ip address
	debug!("Wait to get an ip address");
	let network = Network::new(wifi, current_millis);
	loop {
		network.poll_dhcp().unwrap();

		network.work();

		if network.is_iface_up() {
			info!("got ip {:?}", defmt::Debug2Format(&network.get_ip_info()));
			break;
		}
	}

	let mut i = 0;
	loop {
		trace!("In main(), i was {}", i);
		i += 1;
		yield_now().await
		//Timer::after(Duration::from_millis(1000)).await
	}
}
