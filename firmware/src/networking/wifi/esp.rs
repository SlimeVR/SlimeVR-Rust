use defmt::{error, trace};
use embassy_executor::task;
use embassy_futures::yield_now;
use esp_wifi::{
	create_network_stack_storage, network_stack_storage,
	wifi::utils::create_network_interface,
};

#[task]
pub async fn network_task() {
	let mut storage = create_network_stack_storage!(3, 8, 1, 1);
	let ethernet = create_network_interface(network_stack_storage!(storage));
	let mut wifi = esp_wifi::wifi_interface::Wifi::new(ethernet);

	let mut i = 0;
	loop {
		if let Err(err) = super::connect_wifi(&mut wifi).await {
			error!("Error happened in {:?}", defmt::Debug2Format(&err));
		};
		trace!("In main(), i was {}", i);
		i += 1;
		yield_now().await
		//Timer::after(Duration::from_millis(1000)).await
	}
}
