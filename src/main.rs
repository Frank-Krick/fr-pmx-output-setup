use application::{App, AppFlags};
use iced::Application;

mod application;

fn main() -> std::result::Result<(), iced::Error> {
    let config = fr_pmx_config_lib::read_service_urls();
    App::run(iced::Settings {
        flags: AppFlags {
            port_registry_url: config.pipewire_registry_url,
            pmx_registry_url: config.pmx_registry_url,
        },
        ..Default::default()
    })
}
