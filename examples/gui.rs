use dotenv::dotenv;
use eframe::{Frame, HardwareAcceleration, Renderer, Storage};
use egui::{Context, FontId,  RichText, TextEdit, Ui, Widget};
use kuma_rs::{DataHouse, HouseState, Kuma};
use material_egui::MaterialColors;

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
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
        hardware_acceleration: HardwareAcceleration::Preferred,
        renderer: Renderer::Glow,
        follow_system_theme: true,
        centered: false,

        ..Default::default()
    };

    eframe::run_native("App", options, Box::new(|_cc| Box::from(App::default()))).unwrap();
}

pub type Guard<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone)]
struct App {
    pub api: Kuma,
    pub runtime: Arc<Runtime>,
    pub data: Arc<RwLock<Option<DataHouse>>>,
    pub first_run_ctx: bool,
    pub first_run_gui: bool,
    // false: login
    // true: status
    pub page: bool,
    pub past_state: HouseState,
    pub page_switchable: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            api: Kuma::new("", ""),
            runtime: Arc::new(Runtime::new().unwrap()),
            data: Arc::new(RwLock::new(None)),
            first_run_ctx: true,
            first_run_gui: true,
            past_state: HouseState::Online,
            page: false,
            page_switchable: true,
        }
    }
}

impl App {
    fn request(&self, context: Context) {
        let runtime = self.runtime.clone();
        let engine = self.api.clone();
        let data = self.data.clone();
        let swap = self.page_switchable;

        runtime.spawn(async move {
            loop {
                context.request_repaint();
                if swap {
                    match engine.get().await {
                        Ok(uptime) => {
                            *data.write().unwrap() = Some(uptime);
                        }
                        Err(error) => {
                            println!("{:?}", error);
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.first_run_ctx {
            if let Some((Some(url), Some(auth), Some(page))) = _frame
                .storage()
                .map(|a| (a.get_string("url"), a.get_string("auth"), a.get_string("page").map(|a|a.parse::<bool>().unwrap())))
            {
                self.page = page;
                self.api.url = url;
                self.api.auth = auth;
            };
            self.first_run_ctx = false;
        };
        MaterialColors::new("#F0F".to_string(), true, 1.5).apply_zoom(ctx, &mut self.first_run_ctx);
        egui::CentralPanel::default().show(ctx, |ui| update_fn(self, ui, ctx));
    }
    fn persist_egui_memory(&self) -> bool {
        true
    }
    fn save(&mut self, _storage: &mut dyn Storage) {
        _storage.set_string("url", self.api.url.clone());
        _storage.set_string("auth", self.api.auth.clone());
        _storage.set_string("page", self.page.to_string());
    }
}

fn update_fn(value: &mut App, ui: &mut Ui, ctx: &Context) {
    ui.add_enabled_ui(value.page_switchable, |ui| {
        let str = match value.page.clone() {
            true => "Login",
            false => "Logout",
        };

        ui.checkbox(&mut value.page, str);
    });

    let mut res_url = None;
    let mut res_auth = None;

    if !value.page {
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
        } else if res_auth.unwrap().contains_pointer() {
            if value.api.auth.is_empty() {
                error_message = Some("Token is empty");
            }
        }

        if let Some(msg) = error_message {
            ui.label(msg);
        }

        let can_run = !value.api.auth.is_empty() && !value.api.url.is_empty();
        value.page_switchable = can_run;

        return;
    };

    if value.first_run_gui {
        value.request(ctx.clone());
    };
    if value.first_run_gui {
        value.first_run_gui = false;
    }

    let Ok(data) = value.data.read() else { return };
    let data = match data.is_some() {
        true => data.clone().unwrap(),
        false => {
            ui.label("Data could not be fetched!");
            return;
        }
    };

    ui.label(
        RichText::new(format!("Status: {}", data.state.to_string()))
            .font(FontId::proportional(40.)),
    );
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
        for (name, service) in services {
            ui.collapsing(name, |ui| {
                ui.label(format!("Type: {}", service.monitor_type.to_string()));
                ui.label(format!("URL: {}", service.monitor_url.to_string()));
            });
        }
    }

    // state has changed
    if data.state != value.past_state {
        match data.state {
            HouseState::Online => notify("All services back online"),
            HouseState::Degraded(number) => notify(format!("{number} services offline")),
            HouseState::Offline => notify("All services degraded"),
        }
        value.past_state = data.state
    }
}

pub fn notify(sum: impl Into<String>) {
    use notify_rust::Notification;
    if let Err(error) =  Notification::new()
        .appname("Kuma Desktop")
        .summary(&sum.into())
        .show() {
        println!("{}", error);
    }
}