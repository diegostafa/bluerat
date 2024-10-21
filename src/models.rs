use std::fmt::Display;
use std::str::FromStr;
use std::vec;

use bluer::Address;
use futures::future::join_all;
use itertools::Itertools;
use ratatui::layout::{Alignment, Constraint};
use ratatui::style::{Color, Style};
use ratatui_helpers::stateful_table::Tabular;

use crate::globals::CONFIG;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AdapterId(pub Address);
impl Display for AdapterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DeviceId(pub Address);
impl Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct Adapter {
    pub id: AdapterId,
    pub name: String,
    pub devices: Vec<Device>,
    pub is_on: bool,
    pub is_pairable: bool,
    pub is_discoverable: bool,
    pub is_scanning: bool,
    pub connections: usize,
}
impl Adapter {
    pub async fn from(adapter: bluer::Adapter) -> Self {
        let devices = adapter
            .device_addresses()
            .await
            .unwrap()
            .into_iter()
            .map(|addr| adapter.device(addr).unwrap())
            .map(|d| async move { Device::from(d).await });
        let devices = join_all(devices).await;

        Self {
            id: AdapterId(adapter.address().await.unwrap()),
            name: adapter.name().to_string(),
            is_on: adapter.is_powered().await.unwrap(),
            is_pairable: adapter.is_pairable().await.unwrap(),
            is_discoverable: adapter.is_discoverable().await.unwrap(),
            is_scanning: adapter.is_discovering().await.unwrap(),
            connections: devices.iter().filter(|d| d.is_connected).count(),
            devices,
        }
    }
    pub fn get_info_line(&self) -> String {
        [
            format!("Name: {}", self.name),
            format!("Address: {}", self.id),
        ]
        .into_iter()
        .chain(
            [
                (self.is_discoverable, "Discoverable"),
                (self.is_pairable, "Pairable"),
                (self.is_scanning, "Scanning"),
            ]
            .into_iter()
            .filter(|(f, _)| *f)
            .map(|(_, s)| s.to_string()),
        )
        .map(|s| format!("[{s}]"))
        .join(" | ")
    }
    pub fn get_device(&self, id: &DeviceId) -> Option<&Device> {
        self.devices.iter().find(|d| d.id == *id)
    }
    pub fn get_device_mut(&mut self, id: &DeviceId) -> Option<&mut Device> {
        self.devices.iter_mut().find(|d| d.id == *id)
    }
}
impl Tabular for Adapter {
    type Value = Self;
    fn value(&self) -> Self::Value {
        self.clone()
    }

    fn content(&self) -> Vec<String> {
        let flags = [
            (self.is_discoverable, "Discoverable"),
            (self.is_pairable, "Pairable"),
            (self.is_scanning, "Scanning"),
        ]
        .into_iter()
        .filter(|(f, _)| *f)
        .map(|(_, s)| s.to_string())
        .join(", ");

        vec![
            format!("{}", if self.is_on { "On" } else { "Off" }),
            format!("{}", self.name),
            format!("{}/{}", self.connections, self.devices.len()),
            format!("{}", flags),
        ]
    }
    fn column_constraints() -> Vec<fn(u16) -> Constraint> {
        vec![
            Constraint::Length,
            Constraint::Length,
            Constraint::Length,
            Constraint::Fill,
        ]
    }
    fn column_names() -> Option<Vec<String>> {
        Some(vec![
            "Power".to_string(),
            "Name".to_string(),
            "Connections".to_string(),
            "State".to_string(),
        ])
    }
    fn column_alignments() -> Option<Vec<Alignment>> {
        Some(vec![
            Alignment::Center,
            Alignment::Center,
            Alignment::Center,
            Alignment::Right,
        ])
    }
    fn style(&self) -> Style {
        let mut style = Style::default();
        if self.connections > 0 {
            style = style
                .fg(Color::from_str(&CONFIG.theme.fg_connected_color).unwrap())
                .bg(Color::from_str(&CONFIG.theme.bg_connected_color).unwrap());
        }
        style
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AdapterAction {
    SetPowered(bool),
    SetScanning(bool),
    SetDiscoverable(bool),
    SetPairable(bool),
    Info,
}
impl AdapterAction {
    pub fn shortcut(&self) -> String {
        match self {
            AdapterAction::SetPowered(_) => "p".to_string(),
            AdapterAction::SetScanning(_) => "s".to_string(),
            AdapterAction::SetDiscoverable(_) => "d".to_string(),
            AdapterAction::SetPairable(_) => "p".to_string(),
            AdapterAction::Info => "i".to_string(),
        }
    }
}
impl Display for AdapterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterAction::SetPowered(true) => write!(f, "Power On"),
            AdapterAction::SetPowered(false) => write!(f, "Power Off"),
            AdapterAction::SetScanning(true) => write!(f, "Start Scanning"),
            AdapterAction::SetScanning(false) => write!(f, "Stop Scanning"),
            AdapterAction::SetDiscoverable(true) => write!(f, "Set Discoverable"),
            AdapterAction::SetDiscoverable(false) => write!(f, "Set Not Discoverable"),
            AdapterAction::SetPairable(true) => write!(f, "Set Pairable"),
            AdapterAction::SetPairable(false) => write!(f, "Set Not Pairable"),
            AdapterAction::Info => write!(f, "Info"),
        }
    }
}
impl Tabular for AdapterAction {
    type Value = Self;
    fn value(&self) -> Self::Value {
        *self
    }
    fn content(&self) -> Vec<String> {
        vec![format!("{}", self), format!("{}", self.shortcut())]
    }
    fn column_constraints() -> Vec<fn(u16) -> Constraint> {
        vec![Constraint::Fill, Constraint::Length]
    }
    fn column_alignments() -> Option<Vec<Alignment>> {
        Some(vec![Alignment::Left, Alignment::Right])
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    pub id: DeviceId,
    pub alias: String,
    pub kind: String,
    pub battery: Option<u8>,
    pub is_connected: bool,
    pub is_trusted: bool,
    pub is_paired: bool,
    pub is_blocked: bool,
    pub is_new: bool,
}
impl Device {
    pub async fn from(device: bluer::Device) -> Self {
        Self {
            id: DeviceId(device.address()),
            alias: device.alias().await.unwrap(),
            kind: device
                .icon()
                .await
                .unwrap_or_default()
                .unwrap_or("Unknown".to_string())
                .to_string(),
            battery: device.battery_percentage().await.unwrap(),
            is_connected: device.is_connected().await.unwrap(),
            is_trusted: device.is_trusted().await.unwrap(),
            is_paired: false,
            is_blocked: device.is_blocked().await.unwrap(),
            is_new: false,
        }
    }
    pub async fn from_new(device: bluer::Device) -> Self {
        let mut new = Self::from(device).await;
        new.is_new = true;
        new
    }
}
impl Tabular for Device {
    type Value = Self;
    fn value(&self) -> Self::Value {
        self.clone()
    }
    fn content(&self) -> Vec<String> {
        let battery = self
            .battery
            .map(|b| format!("Battery {b}%"))
            .unwrap_or_default();
        let flags = [
            (self.is_connected, "Connected"),
            (self.is_connected, battery.as_str()),
            (self.is_paired, "Paired"),
            (self.is_blocked, "Blocked"),
            (self.is_trusted, "Trusted"),
            (self.is_new, "New device"),
        ]
        .into_iter()
        .filter(|(f, _)| *f)
        .map(|(_, s)| s.to_string())
        .join(", ");

        vec![
            format!("{}", self.kind),
            format!("{}", self.alias),
            format!("{}", flags),
        ]
    }
    fn column_names() -> Option<Vec<String>> {
        Some(vec![
            "Type".to_string(),
            "Name".to_string(),
            "State".to_string(),
        ])
    }
    fn column_constraints() -> Vec<fn(u16) -> Constraint> {
        vec![Constraint::Length, Constraint::Fill, Constraint::Min]
    }
    fn column_alignments() -> Option<Vec<Alignment>> {
        Some(vec![Alignment::Left, Alignment::Left, Alignment::Right])
    }
    fn style(&self) -> Style {
        let mut style = Style::default();
        if self.is_connected {
            style = style
                .fg(Color::from_str(&CONFIG.theme.fg_connected_color).unwrap())
                .bg(Color::from_str(&CONFIG.theme.bg_connected_color).unwrap());
        }
        if self.is_new {
            style = style
                .fg(Color::from_str(&CONFIG.theme.fg_new_device_color).unwrap())
                .bg(Color::from_str(&CONFIG.theme.bg_new_device_color).unwrap());
        }
        style
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DeviceAction {
    SetConnected(bool),
    SetPaired(bool),
    SetTrusted(bool),
    SetBlocked(bool),
    Info,
}
impl DeviceAction {
    pub fn shortcut(&self) -> String {
        match self {
            DeviceAction::SetConnected(_) => "c".to_string(),
            DeviceAction::SetPaired(true) => "p".to_string(),
            DeviceAction::SetPaired(false) => "r".to_string(),
            DeviceAction::SetTrusted(_) => "t".to_string(),
            DeviceAction::SetBlocked(_) => "b".to_string(),
            DeviceAction::Info => "i".to_string(),
        }
    }
}
impl Display for DeviceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceAction::SetConnected(true) => write!(f, "Connect"),
            DeviceAction::SetConnected(false) => write!(f, "Disconnect"),
            DeviceAction::SetPaired(true) => write!(f, "Pair"),
            DeviceAction::SetPaired(false) => write!(f, "Unpair"),
            DeviceAction::SetTrusted(true) => write!(f, "Trust"),
            DeviceAction::SetTrusted(false) => write!(f, "Untrust"),
            DeviceAction::SetBlocked(true) => write!(f, "Block"),
            DeviceAction::SetBlocked(false) => write!(f, "Unblock"),
            DeviceAction::Info => write!(f, "Info"),
        }
    }
}
impl Tabular for DeviceAction {
    type Value = Self;
    fn value(&self) -> Self::Value {
        *self
    }
    fn content(&self) -> Vec<String> {
        vec![format!("{}", self), format!("{}", self.shortcut())]
    }
    fn column_constraints() -> Vec<fn(u16) -> Constraint> {
        vec![Constraint::Fill, Constraint::Length]
    }
    fn column_alignments() -> Option<Vec<Alignment>> {
        Some(vec![Alignment::Left, Alignment::Right])
    }
}
