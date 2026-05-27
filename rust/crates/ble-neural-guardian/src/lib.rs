//! BLE neural governance client using btleplug.
//! All operations are safe; no unsafe blocks.
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

/// Represents a governed BLE scan + connect operation for neural devices.
pub struct BleNeuralGovernance {
    adapter: Adapter,
    roh_ceiling: u32,
    allowed_service_uuids: Vec<Uuid>,
}

impl BleNeuralGovernance {
    pub async fn new(
        roh_ceiling: u32,
        allowed_service_uuids: Vec<Uuid>,
    ) -> anyhow::Result<Self> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No BLE adapter found"))?;
        Ok(Self {
            adapter,
            roh_ceiling,
            allowed_service_uuids,
        })
    }

    /// Perform a governed scan for neural devices, returning those that advertise
    /// one of the allowed service UUIDs.
    pub async fn scan_for_devices(
        &self,
        scan_duration: Duration,
    ) -> anyhow::Result<Vec<ble_model::BleDeviceObservation>> {
        self.adapter.start_scan(ScanFilter::default()).await?;
        time::sleep(scan_duration).await;
        let peripherals = self.adapter.peripherals().await?;

        let mut observations = Vec::new();
        for p in peripherals {
            let props = p.properties().await?.ok_or_else(|| {
                anyhow::anyhow!("Peripheral properties missing")
            })?;
            let service_uuids: Vec<String> = props
                .services
                .iter()
                .map(|u| u.to_string())
                .collect();
            let has_allowed = service_uuids
                .iter()
                .any(|u| self.allowed_service_uuids.contains(&Uuid::parse_str(u)?));
            if !has_allowed {
                continue;
            }
            observations.push(ble_model::BleDeviceObservation {
                deviceid: props.address.to_string(),
                name: props.local_name.clone(),
                rssidbm: props.rssi.map(|r| r as i16).unwrap_or(0),
                serviceuuids: service_uuids,
                phy: None, // btleplug doesn't expose PHY directly
            });
        }
        self.adapter.stop_scan().await?;
        Ok(observations)
    }

    /// Connect to a device and check RoH ceiling before allowing connection.
    pub async fn governed_connect(
        &self,
        device_id: &str,
        assumed_roh: u32,
    ) -> anyhow::Result<btleplug::platform::Peripheral> {
        if assumed_roh > self.roh_ceiling {
            anyhow::bail!("ROH ceiling exceeded: {} > {}", assumed_roh, self.roh_ceiling);
        }
        let peripherals = self.adapter.peripherals().await?;
        let p = peripherals
            .into_iter()
            .find(|p| p.properties().await.ok().map_or(false, |props| {
                props.map_or(false, |props| props.address.to_string() == device_id)
            }))
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;
        p.connect().await?;
        p.discover_services().await?;
        Ok(p)
    }
}
