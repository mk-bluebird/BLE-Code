#![forbid(unsafe_code)]

use ble_model::{BleIntent, BleLinkParams, BlePhy};

use btleplug::api::{CentralEvent, Peripheral};
use btleplug::api::{CharPropFlags, PeripheralProperties};

/// Map a btleplug CentralEvent into a non-actuating BleIntent candidate.
///
/// This is *descriptive* only. The host decides whether to ask the guard and then act.
pub fn event_to_intent(class_id: &str, event: &CentralEvent) -> Option<BleIntent> {
    match event {
        CentralEvent::DeviceDiscovered(id) => Some(BleIntent::Scan {
            class_id: Some(class_id.to_string()),
        }),
        CentralEvent::DeviceConnected(id) => Some(BleIntent::Connect {
            class_id: class_id.to_string(),
            device_id: id.to_string(),
        }),
        _ => None,
    }
}

/// Derive BleLinkParams from btleplug properties.
///
/// The host should call this *after* a link is established but *before* any guarded op,
/// to capture the negotiated parameters.
pub fn properties_to_link_params(props: &PeripheralProperties) -> BleLinkParams {
    BleLinkParams {
        phy: infer_phy(props),
        encrypted: props.secure_connections.unwrap_or(false),
        mic_present: props.mic_present.unwrap_or(false),
        bonded: props.bonded.unwrap_or(false),
        conn_interval_ms: props.connection_interval.map(|i| i as u32).unwrap_or(50),
        max_pdu_bytes: props.mtu.map(|m| m as u16).unwrap_or(23),
        cte_present: props.cte_present.unwrap_or(false),
    }
}

fn infer_phy(props: &PeripheralProperties) -> BlePhy {
    match props.phy.as_deref() {
        Some("LE2M") => BlePhy::Le2M,
        Some("LECODED_S2") => BlePhy::LeCodedS2,
        Some("LECODED_S8") => BlePhy::LeCodedS8,
        _ => BlePhy::Le1M,
    }
}
