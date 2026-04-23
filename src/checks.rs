use crate::health::ServiceState;
use sysinfo::{Process, System};

#[tracing::instrument(name = "Check CPU load", skip_all)]
pub(crate) async fn get_cpu_load(sys: &System) -> f32 {
    (sys.global_cpu_usage() * 100.0).ceil() / 100.0
}

// check the local RAM usage
#[tracing::instrument(name = "Check RAM load", skip_all)]
pub(crate) async fn get_ram_load(sys: &System) -> f32 {
    ((sys.used_memory() as f32 / sys.total_memory() as f32) * 10000.0).ceil() / 100.0
}

#[tracing::instrument(name = "Check if local service is running", skip_all)]
pub(crate) async fn check_local_service(sys: &System, service_name: &String) -> ServiceState {
    let matchin_process_list: Vec<&Process> =
        sys.processes_by_name(service_name.as_ref()).collect();
    if !matchin_process_list.is_empty() {
        ServiceState::Up
    } else {
        ServiceState::Down
    }
}

#[cfg(test)]
mod tests {

    use sysinfo::System;

    use crate::checks::{get_cpu_load, get_ram_load};

    #[tokio::test]
    async fn cpu_value_between_0_and_100() {
        let system_state = System::new_all();
        let cpu = get_cpu_load(&system_state).await;
        assert!(cpu < 100.0);
        assert!(cpu > 0.0);
    }

    #[tokio::test]
    async fn ram_value_between_0_and_100() {
        let system_state = System::new_all();
        let ram = get_ram_load(&system_state).await;
        assert!(ram < 100.0);
        assert!(ram > 0.0);
    }

    // #[tokio::test]
    // async fn check_service_network_service_is_found() {
    //     let system_state = Arc::new(Mutex::new(System::new_all()));
    //     let service_state = check_local_service(&system_state, &"network".to_string()).await;
    //     assert_eq!(service_state, ServiceState::Up);
    // }
}
