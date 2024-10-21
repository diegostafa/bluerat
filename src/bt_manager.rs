use std::cmp::Ordering;
use std::collections::HashMap;

use itertools::Itertools;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::oneshot::Receiver;
use tokio::task::JoinHandle;

use crate::models::{Adapter, AdapterAction, AdapterId, Device, DeviceAction, DeviceId};

pub enum TaskStatus<T> {
    None,
    Running,
    Error(String),
    Done(T),
}
pub struct BtManager {
    pub session: bluer::Session,
    adapters: HashMap<AdapterId, Adapter>,
    adapter_actions_ch: Option<Receiver<Result<AdapterId, bluer::Error>>>,
    device_actions_ch: Option<Receiver<Result<AdapterId, bluer::Error>>>,
}
impl BtManager {
    pub async fn new() -> Self {
        Self {
            session: bluer::Session::new().await.unwrap(),
            adapters: HashMap::new(),
            adapter_actions_ch: None,
            device_actions_ch: None,
        }
    }
    pub async fn update_adapters(&mut self) {
        self.adapters.clear();
        let adapters = self
            .session
            .adapter_names()
            .await
            .unwrap()
            .into_iter()
            .map(|a| self.session.adapter(&a).unwrap())
            .collect_vec();

        for a in adapters {
            self.adapters.insert(
                AdapterId(a.address().await.unwrap()),
                Adapter::from(a).await,
            );
        }
    }
    pub async fn update_adapter(&mut self, adapter_id: &AdapterId) {
        self.adapters.remove(adapter_id);
        if let Some(adapter) = self.get_actual_adapter(adapter_id).await {
            self.adapters.insert(
                AdapterId(adapter.address().await.unwrap()),
                Adapter::from(adapter).await,
            );
        }
    }

    pub fn mark_new_device(&mut self, device_id: &DeviceId) {
        for a in self.adapters.values_mut() {
            for d in a.devices.iter_mut() {
                if d.id == *device_id {
                    d.is_new = true;
                    return;
                }
            }
        }
    }
    pub fn get_adapters(&self, sorter: &Sorter<Adapter>) -> Vec<Adapter> {
        self.adapters
            .values()
            .cloned()
            .sorted_by(Adapter::BY_ADDRESS.0)
            .sorted_by(sorter.0)
            .collect()
    }
    pub fn get_adapter(&self, adapter_id: &AdapterId) -> Option<&Adapter> {
        self.adapters.get(adapter_id)
    }
    pub fn get_adapter_mut(&mut self, adapter_id: &AdapterId) -> Option<&mut Adapter> {
        self.adapters.get_mut(adapter_id)
    }
    pub fn get_random_adapter(&self) -> Option<&Adapter> {
        self.adapters.values().next()
    }
    pub fn get_devices(&self, adapter_id: &AdapterId, sorter: &Sorter<Device>) -> Vec<Device> {
        self.get_adapter(adapter_id).map_or(Vec::new(), |a| {
            a.devices
                .clone()
                .into_iter()
                .sorted_by(Device::BY_ADDRESS.0)
                .sorted_by(sorter.0)
                .collect()
        })
    }
    pub fn get_device(&self, adapter_id: &AdapterId, device_id: &DeviceId) -> Option<&Device> {
        self.get_adapter(adapter_id)
            .and_then(|a| a.get_device(device_id))
    }
    pub fn get_device_mut(
        &mut self,
        adapter_id: &AdapterId,
        device_id: &DeviceId,
    ) -> Option<&mut Device> {
        self.get_adapter_mut(adapter_id)
            .and_then(|a| a.get_device_mut(device_id))
    }

    pub async fn get_actual_device(
        &self,
        adapter_id: &AdapterId,
        device_id: &DeviceId,
    ) -> Option<bluer::Device> {
        self.get_actual_adapter(adapter_id)
            .await
            .clone()
            .unwrap()
            .device(device_id.0)
            .ok()
    }
    pub async fn get_actual_adapter(&self, adapter_id: &AdapterId) -> Option<bluer::Adapter> {
        let adapters = self
            .session
            .adapter_names()
            .await
            .unwrap()
            .into_iter()
            .map(|a| self.session.adapter(&a).unwrap());

        for a in adapters {
            if a.address().await.unwrap() == adapter_id.0 {
                return Some(a);
            }
        }
        None
    }

    pub async fn exec_adapter_action(
        &mut self,
        adapter_id: &AdapterId,
        action: AdapterAction,
        finally: impl FnOnce() + Send + 'static,
    ) -> Option<JoinHandle<()>> {
        let (s, r) = tokio::sync::oneshot::channel();
        self.adapter_actions_ch = Some(r);
        let adapter = self.get_actual_adapter(adapter_id).await?;

        Some(tokio::spawn(async move {
            let res = match action {
                AdapterAction::SetPowered(v) => adapter.set_powered(v.into()).await,
                AdapterAction::SetDiscoverable(v) => adapter.set_discoverable(v.into()).await,
                AdapterAction::SetPairable(v) => adapter.set_pairable(v.into()).await,
                AdapterAction::SetScanning(_) | AdapterAction::Info => Ok(()),
            };
            let id = AdapterId(adapter.address().await.unwrap());
            let _ = s.send(res.map(|_| id));
            finally();
        }))
    }
    pub async fn poll_exec_adapter_action(&mut self) -> TaskStatus<()> {
        match &mut self.adapter_actions_ch {
            Some(rx) => match rx.try_recv() {
                Err(TryRecvError::Empty) => TaskStatus::Running,
                Err(TryRecvError::Closed) => {
                    self.adapter_actions_ch = None;
                    TaskStatus::Error("Internal error".into())
                }
                Ok(Err(e)) => {
                    self.adapter_actions_ch = None;
                    TaskStatus::Error(e.message)
                }
                Ok(Ok(id)) => {
                    self.adapter_actions_ch = None;
                    self.update_adapter(&id).await;
                    TaskStatus::Done(())
                }
            },
            None => TaskStatus::None,
        }
    }

    pub async fn exec_device_action(
        &mut self,
        adapter_id: &AdapterId,
        device_id: &DeviceId,
        action: DeviceAction,
        finally: impl FnOnce() + Send + 'static,
    ) -> Option<JoinHandle<()>> {
        let (s, r) = tokio::sync::oneshot::channel();
        self.device_actions_ch = Some(r);

        let adapter = self.get_actual_adapter(adapter_id).await?;
        let device = self.get_actual_device(adapter_id, device_id).await?;

        Some(tokio::spawn(async move {
            let res = match action {
                DeviceAction::SetConnected(true) => device.connect().await,
                DeviceAction::SetConnected(false) => device.disconnect().await,
                DeviceAction::SetPaired(true) => device.pair().await,
                DeviceAction::SetPaired(false) => adapter.remove_device(device.address()).await,
                DeviceAction::SetTrusted(val) => device.set_trusted(val).await,
                DeviceAction::SetBlocked(val) => device.set_blocked(val).await,
                DeviceAction::Info => Ok(()),
            };
            let id = AdapterId(adapter.address().await.unwrap());
            let _ = s.send(res.map(|_| id));
            finally();
        }))
    }
    pub async fn poll_exec_device_action(&mut self) -> TaskStatus<()> {
        match &mut self.device_actions_ch {
            Some(rx) => match rx.try_recv() {
                Err(TryRecvError::Empty) => TaskStatus::Running,
                Err(TryRecvError::Closed) => {
                    self.device_actions_ch = None;
                    TaskStatus::Error("Internal error".into())
                }
                Ok(Err(e)) => {
                    self.device_actions_ch = None;
                    TaskStatus::Error(e.message)
                }
                Ok(Ok(id)) => {
                    self.device_actions_ch = None;
                    self.update_adapter(&id).await;
                    TaskStatus::Done(())
                }
            },
            None => TaskStatus::None,
        }
    }
}

pub struct Sorter<T>(pub fn(&T, &T) -> Ordering);
impl<T> Sorter<T> {
    pub const NONE: Sorter<T> = Self(|_, _| Ordering::Equal);
}
impl Adapter {
    pub const BY_ADDRESS: Sorter<Self> = Sorter(|a, b| a.id.0.cmp(&b.id.0));
    pub const BY_NAME: Sorter<Self> = Sorter(|a, b| a.name.cmp(&b.name));
    pub const BY_CONNECTIONS: Sorter<Self> = Sorter(|b, a| a.connections.cmp(&b.connections));
    pub const BY_DEVICES: Sorter<Self> = Sorter(|b, a| a.devices.len().cmp(&b.devices.len()));
    pub const BY_POWER_ON: Sorter<Self> = Sorter(|b, a| a.is_on.cmp(&b.is_on));
}
impl Device {
    pub const BY_ADDRESS: Sorter<Self> = Sorter(|a, b| a.id.0.cmp(&b.id.0));
    pub const BY_NAME: Sorter<Self> = Sorter(|a, b| a.alias.cmp(&b.alias));
    pub const BY_CONNECTED: Sorter<Self> = Sorter(|b, a| a.is_connected.cmp(&b.is_connected));
    pub const BY_BATTERY: Sorter<Self> = Sorter(|a, b| a.battery.cmp(&b.battery));
}
