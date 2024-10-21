use std::io::{self};
use std::ops::Add;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::vec;

use bluer::{AdapterEvent, DeviceEvent, SessionEvent};
use crossterm::event::{self};
use futures::StreamExt;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::{self};
use ratatui::layout::Position;
use ratatui::widgets::TableState;
use ratatui_helpers::keymap::KeyMap;
use ratatui_helpers::status_line::StatusId;
use ratatui_helpers::view_controller::ViewController;
use tokio::sync::oneshot::error::TryRecvError;

use crate::bt_manager::{BtManager, TaskStatus};
use crate::helpers::{try_init_term, try_release_term};
use crate::keymaps::{AppCommand, AppKeyMap};
use crate::models::{Adapter, AdapterAction, AdapterId, DeviceAction, DeviceId};
use crate::views::{
    AdapterActionsView, AdapterView, DeviceActionsView, DeviceView, HelpView, PopupView, QuitView,
};

#[derive(PartialEq)]
pub enum ViewKind {
    Quit,
    AdapterView,
    AdapterActionsView,

    DeviceView,
    DeviceActionsView,
    NotificationView,

    HelpView,
    StatusView,
}

#[derive(Clone, Default, Debug)]
pub enum AppRequest {
    #[default]
    None,
    RefreshViews,
    CloseView,
    OpenHelpView,
    OpenPopupView(String),
    OpenAdaptersView,
    OpenAdapterActionsViewAt(Adapter, Position),
    ExecAdapterAction(Adapter, AdapterAction),
    OpenDevicesView(Adapter),
    OpenDeviceActionsViewAt(Adapter, DeviceId, Position),
    ExecDeviceAction(AdapterId, DeviceId, DeviceAction),
    MonitorDevice(AdapterId, DeviceId),
    Chain(Vec<AppRequest>),
}
impl AppRequest {
    fn or_else<T: FnOnce() -> Self>(self, other: T) -> Self {
        if matches!(self, AppRequest::None) {
            return other();
        }
        self
    }
}
impl Add for AppRequest {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self.clone(), other.clone()) {
            (AppRequest::Chain(mut reqs1), AppRequest::Chain(mut reqs2)) => {
                reqs1.append(&mut reqs2);
                AppRequest::Chain(reqs1)
            }
            (AppRequest::Chain(mut reqs1), _) => {
                reqs1.push(other);
                AppRequest::Chain(reqs1)
            }
            (_, AppRequest::Chain(mut reqs2)) => {
                reqs2.insert(0, self);
                AppRequest::Chain(reqs2)
            }
            (_, _) => AppRequest::Chain(vec![self, other]),
        }
    }
}

pub struct App {
    bt: BtManager,
    vc: ViewController<BtManager, AppRequest, ViewKind>,
    keymap: AppKeyMap,

    session_event_rx: Option<Receiver<SessionEvent>>,
    adapter_event_rx: Option<Receiver<AdapterEvent>>,
    stop_adapter_event_sx: Option<tokio::sync::oneshot::Sender<()>>,
    device_event_rx: Option<Receiver<DeviceEvent>>,
    stop_device_event_sx: Option<tokio::sync::oneshot::Sender<()>>,
}
impl App {
    pub async fn new() -> Self {
        Self {
            bt: BtManager::new().await,
            vc: ViewController::new(Box::new(QuitView), Duration::from_secs(3)),
            keymap: KeyMap::default(),
            session_event_rx: Default::default(),
            adapter_event_rx: Default::default(),
            stop_adapter_event_sx: Default::default(),
            device_event_rx: Default::default(),
            stop_device_event_sx: Default::default(),
        }
    }
    pub async fn init(mut self) -> Self {
        self.monitor_session();
        self.handle_request(AppRequest::RefreshViews).await;

        let req = match self.bt.get_adapters(&Adapter::BY_CONNECTIONS).first() {
            Some(a) => AppRequest::OpenDevicesView(a.clone()),
            _ => AppRequest::OpenAdaptersView,
        };

        self.handle_request(req).await;
        self
    }
    pub async fn run(mut self) -> Result<(), Box<io::Error>> {
        let mut term = try_init_term()?;
        self.vc.curr().set_title();
        while self.vc.is_running() {
            term.draw(|f| self.vc.draw(f, f.area()))?;

            let req = self.handle_view_event().await
                + self.poll_session_event().await
                + self.poll_adapter_event().await
                + self.poll_device_event().await
                + self.poll_pending_tasks().await;

            self.vc.update_status_line();
            self.handle_request(req).await;
        }
        try_release_term(term)
    }

    fn app_update(&mut self, ev: &Event) -> AppRequest {
        match ev {
            Event::Key(ev) => match self.keymap.get_command(ev) {
                None => AppRequest::None,
                Some(cmd) => match cmd {
                    AppCommand::CloseView => AppRequest::CloseView,
                    AppCommand::OpenHelpView => AppRequest::OpenHelpView,
                    AppCommand::RefreshView => AppRequest::RefreshViews,
                },
            },
            _ => AppRequest::None,
        }
    }
    async fn handle_view_event(&mut self) -> AppRequest {
        match event::poll(Duration::from_millis(200)) {
            Ok(true) => {
                let ev = &event::read().unwrap();
                self.app_update(ev)
                    .or_else(|| self.vc.curr_mut().update(ev))
            }
            _ => AppRequest::None,
        }
    }
    async fn poll_session_event(&mut self) -> AppRequest {
        if let Some(rx) = &self.session_event_rx {
            return match rx.try_recv() {
                Ok(ev) => {
                    match ev {
                        SessionEvent::AdapterAdded(_) => {}
                        SessionEvent::AdapterRemoved(_) => {}
                    };
                    self.vc.show_status(format!("{:?}", ev));
                    AppRequest::RefreshViews
                }
                _ => AppRequest::None,
            };
        }
        AppRequest::None
    }
    async fn poll_adapter_event(&mut self) -> AppRequest {
        self.adapter_event_rx
            .as_ref()
            .map_or(AppRequest::None, |rx| match rx.try_recv() {
                Ok(ev) => {
                    match ev {
                        AdapterEvent::DeviceAdded(device_id) => {
                            self.bt.mark_new_device(&DeviceId(device_id));
                        }
                        AdapterEvent::DeviceRemoved(_) => {}
                        AdapterEvent::PropertyChanged(_) => {}
                    };
                    self.vc.show_status(format!("{:?}", ev));
                    AppRequest::RefreshViews
                }
                _ => AppRequest::None,
            })
    }
    async fn poll_device_event(&mut self) -> AppRequest {
        self.device_event_rx
            .as_ref()
            .map_or(AppRequest::None, |rx| match rx.try_recv() {
                Ok(DeviceEvent::PropertyChanged(prop)) => {
                    self.vc.show_status(format!("{:?}", prop));
                    AppRequest::RefreshViews
                }
                _ => AppRequest::None,
            })
    }
    async fn poll_pending_tasks(&mut self) -> AppRequest {
        let r1 = match self.bt.poll_exec_adapter_action().await {
            TaskStatus::Done(_) => AppRequest::RefreshViews,
            TaskStatus::Error(e) => {
                self.vc.show_status(e);
                AppRequest::None
            }
            _ => AppRequest::None,
        };
        let r2 = match self.bt.poll_exec_device_action().await {
            TaskStatus::Done(_) => AppRequest::RefreshViews,
            TaskStatus::Error(e) => {
                self.vc.show_status(e);
                AppRequest::None
            }
            _ => AppRequest::None,
        };
        r1 + r2
    }

    fn monitor_session(&mut self) {
        let session = self.bt.session.clone();
        let (sx, rx) = std::sync::mpsc::channel();
        self.session_event_rx = Some(rx);
        tokio::spawn(async move {
            let mut events = Box::pin(session.events().await.unwrap());
            while let Some(ev) = events.next().await {
                sx.send(ev).unwrap();
            }
        });
    }
    fn monitor_adapter(&mut self, adapter: bluer::Adapter) {
        let (sx, rx) = std::sync::mpsc::channel();
        self.adapter_event_rx = Some(rx);
        let (stop_sx, mut stop_rx) = tokio::sync::oneshot::channel();
        self.stop_adapter_event_sx = Some(stop_sx);

        tokio::spawn(async move {
            let mut events = Box::pin(adapter.discover_devices().await.unwrap());
            while let Some(ev) = events.next().await {
                match stop_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Closed) => return,
                    Err(TryRecvError::Empty) => sx.send(ev).unwrap(),
                }
            }
        });
    }
    fn monitor_device(&mut self, device: bluer::Device) {
        let (sx, rx) = std::sync::mpsc::channel();
        self.device_event_rx = Some(rx);
        let (stop_sx, mut stop_rx) = tokio::sync::oneshot::channel();
        self.stop_device_event_sx = Some(stop_sx);

        tokio::spawn(async move {
            let mut events = Box::pin(device.events().await.unwrap());
            while let Some(ev) = events.next().await {
                match stop_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Closed) => return,
                    Err(TryRecvError::Empty) => sx.send(ev).unwrap(),
                }
            }
        });
    }

    async fn handle_request(&mut self, req: AppRequest) {
        match req {
            AppRequest::None => {}
            AppRequest::CloseView => self.vc.pop(),
            AppRequest::RefreshViews => {
                self.bt.update_adapters().await;
                self.vc.refresh(&self.bt);
            }
            AppRequest::Chain(reqs) => {
                for req in reqs {
                    Box::pin(self.handle_request(req)).await
                }
            }

            AppRequest::OpenHelpView => self.vc.push(Box::new(HelpView::new())),
            AppRequest::OpenPopupView(msg) => self.vc.push(Box::new(PopupView::new(msg))),

            AppRequest::OpenAdaptersView => {
                self.vc.push(Box::new(AdapterView::new(
                    &self.bt,
                    TableState::new().with_selected(0),
                )));
            }
            AppRequest::OpenDevicesView(adapter) => {
                self.vc.push(Box::new(DeviceView::new(
                    adapter.clone(),
                    TableState::new().with_selected(0),
                )));
            }

            AppRequest::OpenAdapterActionsViewAt(adapter, pos) => {
                let actions = vec![
                    AdapterAction::SetPowered(!adapter.is_on),
                    AdapterAction::SetDiscoverable(!adapter.is_discoverable),
                    AdapterAction::SetScanning(!adapter.is_scanning),
                    AdapterAction::SetPairable(!adapter.is_pairable),
                    AdapterAction::Info,
                ];
                self.vc.push(Box::new(AdapterActionsView::new(
                    adapter,
                    actions,
                    TableState::new().with_selected(0),
                    pos,
                )));
            }
            AppRequest::OpenDeviceActionsViewAt(adapter, device_id, pos) => {
                if let Some(device) = adapter.get_device(&device_id) {
                    let actions = vec![
                        DeviceAction::SetConnected(!device.is_connected),
                        DeviceAction::SetTrusted(!device.is_trusted),
                        DeviceAction::SetBlocked(!device.is_blocked),
                        DeviceAction::SetPaired(!device.is_paired),
                        DeviceAction::Info,
                    ];
                    self.vc.push(Box::new(DeviceActionsView::new(
                        adapter,
                        device_id,
                        actions,
                        TableState::new().with_selected(0),
                        pos,
                    )));
                }
            }

            AppRequest::ExecAdapterAction(adapter, action) => {
                match action {
                    AdapterAction::Info => {
                        todo!()
                    }
                    AdapterAction::SetScanning(true) => {
                        self.vc.show_status(action.to_string());
                        let adapter = self.bt.get_actual_adapter(&adapter.id).await.unwrap();
                        self.monitor_adapter(adapter);
                    }
                    AdapterAction::SetScanning(false) => {
                        self.vc.show_status(action.to_string());
                        if let Some(rx) = std::mem::replace(&mut self.stop_adapter_event_sx, None) {
                            rx.send(()).unwrap();
                        }
                    }
                    _ => {
                        let id = self.vc.show_status_always(action.to_string());
                        let on_complete = {
                            let status = self.vc.status().clone();
                            move || status.lock().unwrap().remove(id)
                        };
                        self.bt
                            .exec_adapter_action(&adapter.id, action, on_complete)
                            .await;
                    }
                };
            }
            AppRequest::ExecDeviceAction(adapter_id, device_id, action) => {
                let mut id = StatusId::default();

                if let DeviceAction::Info = action {
                    todo!();
                }
                if let TaskStatus::Running = self.bt.poll_exec_device_action().await {
                    self.vc
                        .show_status_always("Another device operation is running".into());
                    return;
                }
                if let DeviceAction::SetConnected(val) = action {
                    let device = self
                        .bt
                        .get_device(&adapter_id, &device_id)
                        .expect("Failed to get device");
                    let msg = match val {
                        true => "Connecting to",
                        _ => "Disconnecting from",
                    };
                    id = self
                        .vc
                        .show_status_always(format!("{} {}", msg, device.alias));
                }
                let finally = {
                    let status = self.vc.status().clone();
                    move || status.lock().unwrap().remove(id)
                };
                self.bt
                    .exec_device_action(&adapter_id, &device_id, action, finally)
                    .await;
            }

            AppRequest::MonitorDevice(adapter_id, device_id) => {
                let device = self
                    .bt
                    .get_actual_device(&adapter_id, &device_id)
                    .await
                    .unwrap();
                self.vc.show_status(format!("{:?}", req));

                self.monitor_device(device);
            }
        }
    }
}
