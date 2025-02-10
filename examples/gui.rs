#![feature(mpmc_channel)]
use dotenv::dotenv;
use eframe::{Frame, HardwareAcceleration, Renderer, Storage};
use egui::{Context, FontId, RichText, TextEdit, Ui, Widget};
use egui_notify::Toasts;
use kuma_rs::{Data, DataHouse, HouseState, Kuma};
use material_egui::MaterialColors;
use notify_rust::get_server_information;
use std::sync::mpmc::{self, Receiver, Sender};
use std::{sync::LazyLock, time::Duration};
use tokio::runtime::Runtime;
static MIN_WIDTH: f32 = 300.0;
static DEFAULT_WIDTH: f32 = 480.0;
static MIN_HEIGHT: f32 = 480.0;
static DEFAULT_HEIGHT: f32 = 480.0;

fn main() {
    _ = dotenv();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([DEFAULT_WIDTH, DEFAULT_HEIGHT])
            .with_min_inner_size([MIN_WIDTH, MIN_HEIGHT])
            .with_transparent(true),
        vsync: false,
        hardware_acceleration: HardwareAcceleration::Required,
        renderer: Renderer::Glow,
        follow_system_theme: true,
        centered: false,

        ..Default::default()
    };

    eframe::run_native("App", options, Box::new(|_cc| Box::from(App::default()))).unwrap();
}

type Result<T> = core::result::Result<T, anyhow::Error>;

struct App {
    pub api: Kuma,
    pub runtime: Runtime,
    pub data: Option<Result<DataHouse>>,
    pub first_run_ctx: bool,
    pub first_run_gui: bool,
    pub page: bool,
    pub past_state: HouseState,
    pub page_switchable: bool,
    pub toasts: Toasts,
}

impl App {
    fn default() -> Self {
        Self {
            api: Kuma::new(),
            runtime: Runtime::new().expect("Tokio Not Supported for Platform"),
            data: None,
            first_run_ctx: true,
            first_run_gui: true,
            past_state: HouseState::Online,
            page: false,
            page_switchable: true,
            toasts: Toasts::new(),
        }
    }
}

impl App {
    fn request_loop(&self, api: Receiver<Kuma>, data: Sender<Result<DataHouse>>) {
        let clock = 1;

        self.runtime.spawn(async move {
            loop {
                if let Ok(api) = api.try_recv() {
                    println!("aaaaaaa");
                    let temp = data.send(api.get().await);
                    temp.unwrap_or_else(|a| panic!("{}", a));
                    println!("a");
                };
                println!("loop doing nothing");
                tokio::time::sleep(Duration::from_secs(clock)).await;
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.first_run_ctx {
            if let Some((Some(url), Some(auth), Some(page))) = _frame.storage().map(|a| {
                (
                    a.get_string("url"),
                    a.get_string("auth"),
                    a.get_string("page").map(|a| a.parse::<bool>().unwrap()),
                )
            }) {
                self.page = page;
                self.api.url = url;
                self.api.auth = auth;
            };
            self.first_run_ctx = false;
        };
        MaterialColors::new("#642".to_string(), true, 1.5).apply_zoom(ctx, &mut self.first_run_ctx);
        egui::CentralPanel::default().show(ctx, |ui| update_fn(self, ui, ctx));
    }
    fn save(&mut self, _storage: &mut dyn Storage) {
        _storage.set_string("url", self.api.url.clone());
        _storage.set_string("auth", self.api.auth.clone());
        _storage.set_string("page", self.page.to_string());
    }
    fn persist_egui_memory(&self) -> bool {
        true
    }
}

static API: LazyChannel<Kuma> = LazyLock::new(mpmc::channel);
static RES: LazyChannel<Result<DataHouse>> = LazyLock::new(mpmc::channel);
type LazyChannel<T> = LazyLock<(Sender<T>, Receiver<T>), fn() -> (Sender<T>, Receiver<T>)>;

fn update_fn(value: &mut App, ui: &mut Ui, ctx: &Context) {
    if value.first_run_gui {
        value.first_run_gui = false;
        println!("Running update first time");
        value.request_loop(API.1.clone(), RES.0.clone());
    }

    API.0.send(value.api.clone()).unwrap();
    if let Ok(result) = RES.1.try_recv() {
        value.data = Some(result);
    };

    value.toasts.show(ctx);
    ui.add_enabled_ui(value.page_switchable, |ui| {
        if ui
            .button(if value.page { "Login" } else { "Logout" })
            .clicked()
        {
            if let Some(Err(error)) = &value.data {
                value.page = true;
                value.toasts.error(error.to_string());
            } else {
                value.page = !value.page;
            }
        }
    });

    ui.add_space(10.);

    match value.page {
        true => page_login(value, ui),
        false => page_dash(value, ui),
    }
}

fn page_login(value: &mut App, ui: &mut Ui) {
    let mut res_url = None;
    let mut res_auth = None;

    ui.horizontal(|ui| {
        ui.label("URL");
        res_url = Some(ui.text_edit_singleline(&mut value.api.url));
    });

    ui.horizontal(|ui| {
        ui.label("TOKEN");
        res_auth = Some(
            TextEdit::singleline(&mut value.api.auth)
                .password(true)
                .ui(ui),
        );
    });

    let mut error_message = None;
    // url error checking
    if res_url.unwrap().contains_pointer() {
        if value.api.url.is_empty() {
            error_message = Some("URL is empty");
        }
    } else if res_auth.unwrap().contains_pointer() && value.api.auth.is_empty() {
        error_message = Some("Token is empty");
    }

    if let Some(msg) = error_message {
        ui.label(msg);
    }

    let can_run = !value.api.auth.is_empty() && !value.api.url.is_empty();
    value.page_switchable = can_run;
}

fn page_dash(value: &mut App, ui: &mut Ui) {
    if value.first_run_gui {
        if let Err(error) = get_server_information() {
            value
                .toasts
                .warning("DBus error")
                .set_duration(Duration::from_secs(5).into());
            println!("{:?}", error);
        }
        value.first_run_gui = false;
    };

    let Some(Ok(data)) = &value.data else {
        ui.label("No Data Available :/");
        return;
    };

    ui.label(RichText::new(format!("Status: {}", data.state)).font(FontId::proportional(40.)));
    ui.label(match data.state {
        HouseState::Offline => {
            "Yikes! not a single service can be reached (besides the uptime server)".to_string()
        }

        HouseState::Degraded(a) => {
            format!("Degraded means that at least one service are offline, specifically {a} are.")
        }
        HouseState::Online => {
            "This means that all services are online! no need to stress".to_string()
        }
    });

    // display
    if let Some(_service_list) = data.state.is_degraded() {
        let services = data.offline_services();
        let mut services: Vec<(String, Data)> = services.into_iter().collect();
        services.sort();

        for (name, service) in services {
            ui.collapsing(name, |ui| {
                ui.label(format!("Type: {}", service.monitor_type));
                ui.label(format!("URL: {}", service.monitor_url));
            });
        }
    }

    // state has changed
    if data.state != value.past_state {
        match data.state {
            HouseState::Online => notify("All services back online"),
            HouseState::Degraded(number) => notify(format!("{number} services offline")),
            HouseState::Offline => notify("All services Offline"),
        }
        value.past_state = data.state.clone();
    }
}

pub fn notify(sum: impl Into<String>) {
    use notify_rust::Notification;
    if let Err(error) = Notification::new()
        .appname("Kuma Desktop")
        .summary(&sum.into())
        .show()
    {
        println!("{}", error);
    }
}
