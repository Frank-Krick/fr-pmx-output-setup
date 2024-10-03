use iced::application::Application;
use iced::widget::{column, combo_box, row, text, Column, Container, Row, Scrollable};

use fr_pipewire_registry::pipewire_client::PipewireClient;
use fr_pipewire_registry::port::{ListPort, PortDirection};
use fr_pipewire_registry::ListPortsRequest;

use pmx::output::{self, PmxOutput};
use pmx::pmx_registry_client::PmxRegistryClient;
use pmx::{EmptyRequest, UpdateOutputPortAssignmentsRequest};

use tonic::Request;

pub mod pmx {
    tonic::include_proto!("pmx");

    pub mod input {
        tonic::include_proto!("pmx.input");
    }

    pub mod output {
        tonic::include_proto!("pmx.output");
    }

    pub mod plugin {
        tonic::include_proto!("pmx.plugin");
    }

    pub mod channel_strip {
        tonic::include_proto!("pmx.channel_strip");
    }

    pub mod looper {
        tonic::include_proto!("pmx.looper");
    }
}

pub mod fr_pipewire_registry {

    tonic::include_proto!("pmx.pipewire");

    pub mod node {
        tonic::include_proto!("pmx.pipewire.node");
    }

    pub mod port {
        tonic::include_proto!("pmx.pipewire.port");
    }

    pub mod application {
        tonic::include_proto!("pmx.pipewire.application");
    }

    pub mod device {
        tonic::include_proto!("pmx.pipewire.device");
    }

    pub mod link {
        tonic::include_proto!("pmx.pipewire.link");
    }
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    LoadInputsCompleted((Vec<PmxOutput>, Vec<ListPort>)),
    LeftPortSelected((u32, String)),
    RightPortSelected((u32, String)),
    PortSaved(u32),
}

#[derive(Default, Clone)]
pub struct AppFlags {
    pub port_registry_url: String,
    pub pmx_registry_url: String,
}

#[derive(Debug, Clone)]
struct MixerOutput {
    pmx_output_id: u32,
    name: String,
    selected_left_in_port_path: Option<String>,
    selected_right_in_port_path: Option<String>,
    saved: bool,
}

impl MixerOutput {
    fn from(output: &PmxOutput) -> Self {
        MixerOutput {
            pmx_output_id: output.id,
            name: output.name.clone(),
            selected_left_in_port_path: output.left_port_path.clone(),
            selected_right_in_port_path: output.right_port_path.clone(),
            saved: true,
        }
    }
}

pub struct App {
    outputs: Vec<MixerOutput>,
    pipewire_out_port_paths: iced::widget::combo_box::State<String>,
    pipewire_in_port_paths: iced::widget::combo_box::State<String>,
    flags: Flags,
}

type Executor = iced::executor::Default;
type Message = AppMessage;
type Theme = iced::Theme;
type Flags = AppFlags;

impl App {
    async fn update_output_if_valid(output: MixerOutput, registry_url: String) {
        let mut client = PmxRegistryClient::connect(registry_url).await.unwrap();
        let request = Request::new(UpdateOutputPortAssignmentsRequest {
            id: output.pmx_output_id,
            left_port_path: output.selected_left_in_port_path,
            right_port_path: output.selected_right_in_port_path,
        });
        client
            .update_output_port_assignments(request)
            .await
            .unwrap();
    }
}

impl Application for App {
    type Executor = Executor;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn theme(&self) -> Self::Theme {
        iced::Theme::GruvboxDark
    }
    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let cloned = flags.clone();
        (
            App {
                outputs: Vec::new(),
                pipewire_out_port_paths: iced::widget::combo_box::State::new(Vec::new()),
                pipewire_in_port_paths: iced::widget::combo_box::State::new(Vec::new()),
                flags: cloned,
            },
            iced::Command::perform(
                async move {
                    let mut client = PmxRegistryClient::connect(flags.pmx_registry_url)
                        .await
                        .unwrap();
                    let request = Request::new(EmptyRequest {});
                    let outputs_response = client.list_outputs(request).await.unwrap();

                    let mut client = PipewireClient::connect(flags.port_registry_url)
                        .await
                        .unwrap();
                    let request = Request::new(ListPortsRequest {
                        node_id_filter: None,
                    });
                    let ports_respose = client.list_ports(request).await.unwrap();
                    (
                        outputs_response.get_ref().outputs.clone(),
                        ports_respose.get_ref().ports.clone(),
                    )
                },
                Message::LoadInputsCompleted,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("PMX-1 Output Setup")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::LoadInputsCompleted(inputs) => {
                let in_port_paths = inputs
                    .1
                    .iter()
                    .filter(|p| p.direction == PortDirection::In as i32)
                    .map(|p| p.path.clone())
                    .collect();

                self.pipewire_in_port_paths = iced::widget::combo_box::State::new(in_port_paths);

                let out_port_paths = inputs
                    .1
                    .iter()
                    .filter(|p| p.direction == PortDirection::Out as i32)
                    .map(|p| p.path.clone())
                    .collect();

                self.pipewire_out_port_paths = iced::widget::combo_box::State::new(out_port_paths);

                self.outputs = inputs.0.iter().map(MixerOutput::from).collect();

                iced::Command::none()
            }
            AppMessage::LeftPortSelected((id, path)) => {
                let output = self
                    .outputs
                    .iter_mut()
                    .find(|o| o.pmx_output_id == id)
                    .unwrap();
                output.selected_left_in_port_path = Some(path);
                let registry_url = self.flags.pmx_registry_url.clone();
                let output = output.clone();
                iced::Command::perform(
                    async move {
                        let id = output.pmx_output_id;
                        App::update_output_if_valid(output, registry_url).await;
                        id
                    },
                    Message::PortSaved,
                )
            }
            AppMessage::RightPortSelected((id, path)) => {
                let output = self
                    .outputs
                    .iter_mut()
                    .find(|o| o.pmx_output_id == id)
                    .unwrap();
                output.selected_right_in_port_path = Some(path);
                let registry_url = self.flags.pmx_registry_url.clone();
                let output = output.clone();
                iced::Command::perform(
                    async move {
                        let id = output.pmx_output_id;
                        App::update_output_if_valid(output, registry_url).await;
                        id
                    },
                    Message::PortSaved,
                )
            }
            AppMessage::PortSaved(_) => iced::Command::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let controls = self
            .outputs
            .clone()
            .into_iter()
            .map(|i| {
                Container::new(column![
                    row![text(i.name.clone())
                        .width(125)
                        .height(35)
                        .vertical_alignment(iced::alignment::Vertical::Center),]
                    .padding(5),
                    row![
                        text(String::from("Left"))
                            .width(125)
                            .height(35)
                            .vertical_alignment(iced::alignment::Vertical::Center),
                        combo_box(
                            &self.pipewire_in_port_paths,
                            "Select port",
                            i.selected_left_in_port_path.as_ref(),
                            move |path| AppMessage::LeftPortSelected((i.pmx_output_id, path))
                        )
                        .width(500)
                        .padding(5)
                    ],
                    row![
                        text(String::from("Right"))
                            .width(125)
                            .height(35)
                            .vertical_alignment(iced::alignment::Vertical::Center),
                        combo_box(
                            &self.pipewire_in_port_paths,
                            "Select port",
                            i.selected_right_in_port_path.as_ref(),
                            move |path| AppMessage::RightPortSelected((i.pmx_output_id, path))
                        )
                        .width(500)
                        .padding(5)
                    ],
                ])
                .into()
            })
            .collect();

        Scrollable::new(Column::from_vec(controls).padding(5).spacing(10)).into()
    }
}
