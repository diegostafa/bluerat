use std::vec;

use ratatui::crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, TableState};
use ratatui::Frame;
use ratatui_helpers::keymap::{KeyMap, ShortCut};
use ratatui_helpers::stateful_table::{IndexedRow, InteractiveTable, StatefulTable};
use ratatui_helpers::view::View;

use crate::app::{AppRequest, ViewKind};
use crate::bt_manager::BtManager;
use crate::helpers::centered_rect;
use crate::keymaps::{
    AdapterViewCommand, AdapterViewKeyMap, AppCommand, AppKeyMap, DeviceViewCommand,
    DeviceViewKeyMap,
};
use crate::models::{Adapter, AdapterAction, Device, DeviceAction, DeviceId};
use crate::theme::StyledWidget;

pub struct QuitView;
impl View for QuitView {
    type Model = BtManager;
    type Signal = AppRequest;
    type Kind = ViewKind;
    fn kind(&self) -> ViewKind {
        ViewKind::Quit
    }
    fn set_title(&self) {}
}

pub struct AdapterView<'a> {
    table: StatefulTable<'a, Adapter>,
    keymap: AdapterViewKeyMap,
}
impl AdapterView<'_> {
    pub fn new(bt: &BtManager, state: TableState) -> Self {
        Self {
            table: StyledWidget::table(
                bt.get_adapters(&Adapter::BY_NAME),
                state,
                Some("Adapters".into()),
            ),
            keymap: KeyMap::default(),
        }
    }
}
impl View for AdapterView<'_> {
    type Model = BtManager;
    type Signal = AppRequest;
    type Kind = ViewKind;
    fn is_floating(&self) -> bool {
        true
    }
    fn compute_area(&self, area: Rect) -> Rect {
        let (min_width, min_height) = self.table.min_area();
        centered_rect(area, (min_width, min_height))
    }
    fn kind(&self) -> ViewKind {
        ViewKind::AdapterView
    }
    fn title(&self) -> String {
        "bluerat - adapters".to_string()
    }
    fn refresh(&mut self, model: &Self::Model) {
        *self = Self::new(model, self.table.state().clone());
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.table.draw(f, area);
    }
    fn update(&mut self, ev: &Event) -> AppRequest {
        self.table.update(ev);

        match ev {
            Event::Key(ev) => {
                if let Some(cmd) = self.keymap.get_command(ev) {
                    match cmd {
                        AdapterViewCommand::TogglePower => {
                            if let Some(adapter) = self.table.selected_value() {
                                return AppRequest::ExecAdapterAction(
                                    adapter.clone(),
                                    AdapterAction::SetPowered(!adapter.is_on),
                                );
                            }
                        }
                        AdapterViewCommand::ToggleScan => {
                            if let Some(adapter) = self.table.selected_value() {
                                return AppRequest::ExecAdapterAction(
                                    adapter.clone(),
                                    AdapterAction::SetScanning(!adapter.is_scanning),
                                );
                            }
                        }
                        AdapterViewCommand::OpenMenu => {
                            if let Some(adapter) = self.table.selected_value() {
                                return AppRequest::OpenAdapterActionsViewAt(
                                    adapter.clone(),
                                    (0, 0).into(),
                                );
                            }
                        }
                        AdapterViewCommand::Info => {
                            if let Some(adapter) = self.table.selected_value() {
                                return AppRequest::ExecAdapterAction(
                                    adapter.clone(),
                                    AdapterAction::Info,
                                );
                            }
                        }
                        AdapterViewCommand::OpenDevices => {
                            if let Some(adapter) = self.table.selected_value() {
                                return AppRequest::CloseView
                                    + AppRequest::OpenDevicesView(adapter.clone());
                            };
                        }
                        AdapterViewCommand::TogglePairable => {
                            if let Some(adapter) = self.table.selected_value() {
                                return AppRequest::ExecAdapterAction(
                                    adapter.clone(),
                                    AdapterAction::SetPairable(!adapter.is_pairable),
                                );
                            }
                        }
                        AdapterViewCommand::ToggleDiscoverable => {
                            if let Some(adapter) = self.table.selected_value() {
                                return AppRequest::ExecAdapterAction(
                                    adapter.clone(),
                                    AdapterAction::SetDiscoverable(!adapter.is_discoverable),
                                );
                            }
                        }
                    }
                }
            }
            Event::Mouse(ev) => {
                let pos = (ev.row, ev.column);
                match ev.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if let Some(row) = self.table.screen_coords_to_row_index(pos)
                            && let Some(idx) = self.table.selected_index()
                            && row == idx
                            && let Some(adapter) = self.table.selected_value()
                        {
                            return AppRequest::CloseView
                                + AppRequest::OpenDevicesView(adapter.clone());
                        }
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        if let Some(row) = self.table.screen_coords_to_row_index(pos)
                            && let Some(idx) = self.table.selected_index()
                            && row == idx
                            && let Some(adapter) = self.table.selected_value()
                        {
                            return AppRequest::OpenAdapterActionsViewAt(
                                adapter.clone(),
                                (pos.1, pos.0 + 1).into(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        AppRequest::None
    }
}

pub struct AdapterActionsView<'a> {
    adapter: Adapter,
    table: StatefulTable<'a, AdapterAction>,
    pos: Position,
    area: Rect,
}
impl AdapterActionsView<'_> {
    pub fn new(
        adapter: Adapter,
        actions: Vec<AdapterAction>,
        state: TableState,
        pos: Position,
    ) -> Self {
        Self {
            adapter,
            table: StyledWidget::table(actions, state, None),
            pos,
            area: Rect::default(),
        }
    }
}
impl View for AdapterActionsView<'_> {
    type Model = BtManager;
    type Signal = AppRequest;
    type Kind = ViewKind;
    fn kind(&self) -> Self::Kind {
        ViewKind::AdapterActionsView
    }
    fn is_floating(&self) -> bool {
        true
    }
    fn compute_area(&self, area: Rect) -> Rect {
        let (width, height) = self.table.min_area();
        let (width, height) = (width.min(area.width), height.min(area.height));
        let x = area.width.saturating_sub(width).min(self.pos.x);
        let y = area.height.saturating_sub(height).min(self.pos.y);
        Rect {
            x,
            y,
            width,
            height,
        }
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.area = area;
        self.table.draw(f, area);
    }
    fn update(&mut self, ev: &Event) -> AppRequest {
        self.table.update(ev);
        match ev {
            Event::Key(ev) => match ev.code {
                KeyCode::Enter => {
                    if let Some(value) = self.table.selected_value() {
                        return AppRequest::CloseView
                            + AppRequest::ExecAdapterAction(self.adapter.clone(), *value);
                    };
                }
                _ => {}
            },
            Event::Mouse(ev) => {
                let pos = (ev.row, ev.column);

                match ev.kind {
                    MouseEventKind::Down(MouseButton::Left | MouseButton::Right) => {
                        if !self.area.contains(Position { x: pos.1, y: pos.0 }) {
                            return AppRequest::CloseView;
                        }

                        if self.table.screen_coords_to_row_index(pos).is_some()
                            && let Some(value) = self.table.selected_value()
                        {
                            return AppRequest::CloseView
                                + AppRequest::ExecAdapterAction(self.adapter.clone(), *value);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        AppRequest::None
    }
}

pub struct DeviceView<'a> {
    adapter: Adapter,
    adapter_info: Paragraph<'a>,
    table: StatefulTable<'a, IndexedRow<Device>>,
    layout: Layout,
    keymap: DeviceViewKeyMap,
}
impl DeviceView<'_> {
    pub fn new(adapter: Adapter, state: TableState) -> Self {
        Self {
            table: StyledWidget::indexed_table(
                adapter.devices.clone(),
                state,
                Some("Devices".into()),
            ),
            adapter_info: Paragraph::new(adapter.get_info_line())
                .block(StyledWidget::block().title("Adapter".to_string())),
            layout: Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Fill(1)]),
            adapter,
            keymap: KeyMap::default(),
        }
    }
}
impl View for DeviceView<'_> {
    type Model = BtManager;
    type Signal = AppRequest;
    type Kind = ViewKind;
    fn kind(&self) -> ViewKind {
        ViewKind::DeviceView
    }
    fn title(&self) -> String {
        "bluerat - devices".to_string()
    }
    fn refresh(&mut self, model: &Self::Model) {
        if let Some(adapter) = model.get_adapter(&self.adapter.id) {
            *self = Self::new(adapter.clone(), self.table.state().clone());
        } else if let Some(adapter) = model.get_random_adapter() {
            *self = Self::new(adapter.clone(), self.table.state().clone());
        } else {
            self.table = StyledWidget::indexed_table(
                vec![],
                self.table.state().clone(),
                Some("Devices".into()),
            );
            self.adapter_info = Paragraph::new("No adapters found".to_string());
        }
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let layout = self.layout.split(area);
        f.render_widget(self.adapter_info.clone(), layout[0]);
        self.table.draw(f, layout[1]);
    }
    fn update(&mut self, ev: &Event) -> AppRequest {
        self.table.update(ev);
        match ev {
            Event::Key(ev) => {
                if let Some(cmd) = self.keymap.get_command(ev) {
                    match cmd {
                        DeviceViewCommand::ToggleConnect => {
                            if let Some(device) = self.table.selected_value() {
                                return AppRequest::ExecDeviceAction(
                                    self.adapter.id,
                                    device.id,
                                    DeviceAction::SetConnected(!device.is_connected),
                                );
                            }
                        }
                        DeviceViewCommand::Pair => {
                            if let Some(device) = self.table.selected_value() {
                                return AppRequest::ExecDeviceAction(
                                    self.adapter.id,
                                    device.id,
                                    DeviceAction::SetPaired(!device.is_paired),
                                );
                            }
                        }
                        DeviceViewCommand::ToggleBlock => {
                            if let Some(device) = self.table.selected_value() {
                                return AppRequest::ExecDeviceAction(
                                    self.adapter.id,
                                    device.id,
                                    DeviceAction::SetBlocked(!device.is_blocked),
                                );
                            }
                        }
                        DeviceViewCommand::ToggleTrust => {
                            if let Some(device) = self.table.selected_value() {
                                return AppRequest::ExecDeviceAction(
                                    self.adapter.id,
                                    device.id,
                                    DeviceAction::SetTrusted(!device.is_trusted),
                                );
                            }
                        }
                        DeviceViewCommand::OpenMenu => {
                            if let Some(device) = self.table.selected_value() {
                                return AppRequest::OpenDeviceActionsViewAt(
                                    self.adapter.clone(),
                                    device.id,
                                    (0, 0).into(),
                                );
                            }
                        }

                        DeviceViewCommand::Info => {
                            if let Some(device) = self.table.selected_value() {
                                return AppRequest::MonitorDevice(self.adapter.id, device.id);
                            }
                        }
                        DeviceViewCommand::Unpair => {}
                        DeviceViewCommand::ShowAdapters => return AppRequest::OpenAdaptersView,
                        DeviceViewCommand::ToggleScan => {
                            return AppRequest::ExecAdapterAction(
                                self.adapter.clone(),
                                AdapterAction::SetScanning(!self.adapter.is_scanning),
                            )
                        }
                        DeviceViewCommand::Monitor => {
                            if let Some(device) = self.table.selected_value() {
                                return AppRequest::MonitorDevice(self.adapter.id, device.id);
                            }
                        }
                    }
                }
            }
            Event::Mouse(ev) => {
                let pos = (ev.row, ev.column);
                match ev.kind {
                    MouseEventKind::Down(MouseButton::Right) => {
                        if let Some(row) = self.table.screen_coords_to_row_index(pos)
                            && let Some(idx) = self.table.selected_index()
                            && row == idx
                            && let Some(device) = self.table.selected_value()
                        {
                            return AppRequest::OpenDeviceActionsViewAt(
                                self.adapter.clone(),
                                device.id,
                                (pos.1, pos.0 + 1).into(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        AppRequest::None
    }
}

pub struct DeviceActionsView<'a> {
    adapter: Adapter,
    device_id: DeviceId,
    table: StatefulTable<'a, DeviceAction>,
    pos: Position,
    area: Rect,
}
impl DeviceActionsView<'_> {
    pub fn new(
        adapter: Adapter,
        device_id: DeviceId,
        actions: Vec<DeviceAction>,
        state: TableState,
        pos: Position,
    ) -> Self {
        Self {
            adapter: adapter,
            device_id,
            table: StyledWidget::table(actions, state, None),
            pos,
            area: Rect::default(),
        }
    }
}
impl View for DeviceActionsView<'_> {
    type Model = BtManager;
    type Signal = AppRequest;
    type Kind = ViewKind;
    fn kind(&self) -> ViewKind {
        ViewKind::DeviceActionsView
    }
    fn is_floating(&self) -> bool {
        true
    }
    fn compute_area(&self, area: Rect) -> Rect {
        let (width, height) = self.table.min_area();
        let (width, height) = (width.min(area.width), height.min(area.height));
        let x = area.width.saturating_sub(width).min(self.pos.x);
        let y = area.height.saturating_sub(height).min(self.pos.y);
        Rect {
            x,
            y,
            width,
            height,
        }
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.area = area;
        self.table.draw(f, area);
    }
    fn update(&mut self, ev: &Event) -> AppRequest {
        self.table.update(ev);

        match ev {
            Event::Key(ev) => match ev.code {
                KeyCode::Char('r') => return AppRequest::RefreshViews,
                KeyCode::Enter => {
                    if let Some(value) = self.table.selected_value() {
                        return AppRequest::CloseView
                            + AppRequest::ExecDeviceAction(
                                self.adapter.id,
                                self.device_id,
                                *value,
                            );
                    };
                }
                _ => {}
            },
            Event::Mouse(ev) => {
                let pos = (ev.row, ev.column);

                match ev.kind {
                    MouseEventKind::Down(MouseButton::Left | MouseButton::Right) => {
                        if !self.area.contains(Position { x: pos.1, y: pos.0 }) {
                            return AppRequest::CloseView;
                        }

                        if self.table.screen_coords_to_row_index(pos).is_some()
                            && let Some(value) = self.table.selected_value()
                        {
                            return AppRequest::CloseView
                                + AppRequest::ExecDeviceAction(
                                    self.adapter.id,
                                    self.device_id,
                                    *value,
                                );
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        AppRequest::None
    }
}

pub struct HelpView<'a> {
    app_table: StatefulTable<'a, ShortCut<AppCommand>>,
    adapter_table: StatefulTable<'a, ShortCut<AdapterViewCommand>>,
    device_table: StatefulTable<'a, ShortCut<DeviceViewCommand>>,
    layout: Layout,
}
impl HelpView<'_> {
    pub fn new() -> Self {
        Self {
            app_table: StyledWidget::table(
                AppKeyMap::default().0,
                TableState::default(),
                Some("Global Shortcuts".into()),
            ),
            adapter_table: StyledWidget::table(
                AdapterViewKeyMap::default().0,
                TableState::default(),
                Some("Shortcuts for adapters".into()),
            ),
            device_table: StyledWidget::table(
                DeviceViewKeyMap::default().0,
                TableState::default(),
                Some("Shortcuts for devices".into()),
            ),
            layout: Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ]),
        }
    }
}
impl View for HelpView<'_> {
    type Model = BtManager;
    type Signal = AppRequest;
    type Kind = ViewKind;
    fn kind(&self) -> ViewKind {
        ViewKind::HelpView
    }
    fn update(&mut self, ev: &Event) -> Self::Signal {
        self.adapter_table.update(ev);
        self.device_table.update(ev);
        self.app_table.update(ev);
        Self::Signal::default()
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let layout = self.layout.split(area);
        self.app_table.draw(f, layout[0]);
        self.adapter_table.draw(f, layout[1]);
        self.device_table.draw(f, layout[2]);
    }
}

pub struct PopupView<'a> {
    p: Paragraph<'a>,
}
impl PopupView<'_> {
    pub fn new(msg: String) -> Self {
        Self {
            p: Paragraph::new(msg).block(Block::default().borders(Borders::ALL)),
        }
    }
}
impl View for PopupView<'_> {
    type Model = BtManager;
    type Signal = AppRequest;
    type Kind = ViewKind;
    fn kind(&self) -> ViewKind {
        ViewKind::NotificationView
    }
    fn compute_area(&self, area: Rect) -> Rect {
        let (width, height) = (50, 15);
        let (width, height) = (width.min(area.width), height.min(area.height));
        centered_rect(area, (width, height))
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        f.render_widget(&self.p, area);
    }
    fn is_floating(&self) -> bool {
        true
    }
}
