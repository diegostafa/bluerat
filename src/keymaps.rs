use std::collections::HashMap;
use std::fmt::Display;
use std::vec;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_helpers::keymap::{KeyMap, ShortCut};
use ratatui_helpers::stateful_table::TableKeyMap;

pub enum AppCommand {
    CloseView,
    OpenHelpView,
    RefreshView,
}
impl Display for AppCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppCommand::CloseView => write!(f, "quit view"),
            AppCommand::OpenHelpView => write!(f, "help"),
            AppCommand::RefreshView => write!(f, "refresh"),
        }
    }
}
pub struct AppKeyMap(pub Vec<ShortCut<AppCommand>>);
impl KeyMap for AppKeyMap {
    type Command = AppCommand;
    fn get_shortcuts(&self) -> &[ShortCut<Self::Command>] {
        &self.0
    }
    fn default() -> Self {
        Self(Vec::from([
            ShortCut(
                AppCommand::CloseView,
                vec![
                    KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                    KeyEvent::new(KeyCode::Left, KeyModifiers::ALT),
                    KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                ],
            ),
            ShortCut(
                AppCommand::OpenHelpView,
                vec![
                    KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
                    KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                ],
            ),
            ShortCut(
                AppCommand::RefreshView,
                vec![KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
            ),
        ]))
    }
}

pub enum AdapterViewCommand {
    TogglePower,
    ToggleScan,
    TogglePairable,
    ToggleDiscoverable,
    OpenMenu,
    OpenDevices,
    Info,
}
impl Display for AdapterViewCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterViewCommand::TogglePower => write!(f, "toggle power"),
            AdapterViewCommand::ToggleScan => write!(f, "toggle scan"),
            AdapterViewCommand::OpenMenu => write!(f, "open menu"),
            AdapterViewCommand::Info => write!(f, "info"),
            AdapterViewCommand::OpenDevices => write!(f, "open devices"),
            AdapterViewCommand::TogglePairable => write!(f, "toggle pairable"),
            AdapterViewCommand::ToggleDiscoverable => write!(f, "toggle discoverable"),
        }
    }
}
pub struct AdapterViewKeyMap(pub Vec<ShortCut<AdapterViewCommand>>);
impl KeyMap for AdapterViewKeyMap {
    type Command = AdapterViewCommand;
    fn get_shortcuts(&self) -> &[ShortCut<Self::Command>] {
        &self.0
    }
    fn default() -> Self {
        Self(Vec::from([
            ShortCut(
                AdapterViewCommand::TogglePower,
                vec![KeyEvent::new(KeyCode::Char('P'), KeyModifiers::SHIFT)],
            ),
            ShortCut(
                AdapterViewCommand::ToggleDiscoverable,
                vec![KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)],
            ),
            ShortCut(
                AdapterViewCommand::TogglePairable,
                vec![KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)],
            ),
            ShortCut(
                AdapterViewCommand::ToggleScan,
                vec![KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)],
            ),
            ShortCut(
                AdapterViewCommand::OpenMenu,
                vec![KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE)],
            ),
            ShortCut(
                AdapterViewCommand::OpenDevices,
                vec![KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)],
            ),
            ShortCut(
                AdapterViewCommand::Info,
                vec![KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE)],
            ),
        ]))
    }
}

pub enum DeviceViewCommand {
    ToggleConnect,
    ToggleTrust,
    ToggleBlock,
    ToggleScan,
    Pair,
    Unpair,
    OpenMenu,
    Info,
    ShowAdapters,
    Monitor,
}
impl Display for DeviceViewCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceViewCommand::ToggleConnect => write!(f, "toggle connect"),
            DeviceViewCommand::ToggleBlock => write!(f, "toggle block"),
            DeviceViewCommand::ToggleTrust => write!(f, "toggle trust"),
            DeviceViewCommand::ToggleScan => write!(f, "toggle scan"),
            DeviceViewCommand::Pair => write!(f, "pair"),
            DeviceViewCommand::Unpair => write!(f, "unpair"),
            DeviceViewCommand::OpenMenu => write!(f, "open menu"),
            DeviceViewCommand::Info => write!(f, "info"),
            DeviceViewCommand::ShowAdapters => write!(f, "show adapters"),
            DeviceViewCommand::Monitor => write!(f, "monitor"),
        }
    }
}
pub struct DeviceViewKeyMap(pub Vec<ShortCut<DeviceViewCommand>>);
impl KeyMap for DeviceViewKeyMap {
    type Command = DeviceViewCommand;
    fn get_shortcuts(&self) -> &[ShortCut<Self::Command>] {
        &self.0
    }
    fn default() -> Self {
        Self(Vec::from([
            ShortCut(
                DeviceViewCommand::ToggleScan,
                vec![KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)],
            ),
            ShortCut(
                DeviceViewCommand::ToggleConnect,
                vec![KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)],
            ),
            ShortCut(
                DeviceViewCommand::ToggleBlock,
                vec![KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)],
            ),
            ShortCut(
                DeviceViewCommand::Pair,
                vec![KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)],
            ),
            ShortCut(
                DeviceViewCommand::Unpair,
                vec![KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE)],
            ),
            ShortCut(
                DeviceViewCommand::OpenMenu,
                vec![
                    KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
                    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                ],
            ),
            ShortCut(
                DeviceViewCommand::Info,
                vec![KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE)],
            ),
            ShortCut(
                DeviceViewCommand::ShowAdapters,
                vec![
                    KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
                    KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
                ],
            ),
            ShortCut(
                DeviceViewCommand::Monitor,
                vec![KeyEvent::new(KeyCode::Char('m'), KeyModifiers::SHIFT)],
            ),
        ]))
    }
}

pub fn get_keymap_collisions() -> Vec<(KeyEvent, Vec<String>)> {
    let mut map: HashMap<KeyEvent, Vec<String>> = HashMap::new();
    for sc in AppKeyMap::default().0 {
        for key in sc.1 {
            map.entry(key).or_default().push(sc.0.to_string());
        }
    }
    for sc in AdapterViewKeyMap::default().0 {
        for key in sc.1 {
            map.entry(key).or_default().push(sc.0.to_string());
        }
    }
    for sc in DeviceViewKeyMap::default().0 {
        for key in sc.1 {
            map.entry(key).or_default().push(sc.0.to_string());
        }
    }
    for sc in TableKeyMap::default().0 {
        for key in sc.1 {
            map.entry(key).or_default().push(sc.0.to_string());
        }
    }
    map.into_iter().filter(|(_, v)| v.len() > 1).collect()
}
