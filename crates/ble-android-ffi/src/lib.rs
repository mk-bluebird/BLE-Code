#![forbid(unsafe_code)]

use ble_governance::BleProfileShard;
use ble_guard::{BleGuard, BleGuardDecision};
use ble_model::{BleIntent, BleLinkParams};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;

use crate::ffi_safety::str_from_ptr;

/// Global guard instance for the process.
static GUARD: OnceCell<Mutex<BleGuard>> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FfiIntent {
    Scan,
    Connect {
        class_id: String,
        device_id: String,
    },
    SubscribeCharacteristic {
        class_id: String,
        device_id: String,
        service_uuid: String,
        characteristic_uuid: String,
    },
    WriteCharacteristic {
        class_id: String,
        device_id: String,
        service_uuid: String,
        characteristic_uuid: String,
        payload_len: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FfiLinkParams {
    pub phy: String,
    pub encrypted: bool,
    pub mic_present: bool,
    pub bonded: bool,
    pub conn_interval_ms: u32,
    pub max_pdu_bytes: u16,
    pub cte_present: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BleGuardRequest {
    pub intent: FfiIntent,
    pub link: Option<FfiLinkParams>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BleGuardResponse {
    pub decision: String, // "allowed" | "rejected"
    pub reason: Option<String>,
}

/// Initialize the global guard with a serialized BleProfileShard (JSON).
#[no_mangle]
pub extern "C" fn init_ble_guard_from_profile_json(json: *const c_char) -> bool {
    let json_str = match str_from_ptr(json) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let shard: BleProfileShard = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let guard = BleGuard::new(shard);
    GUARD.set(Mutex::new(guard)).is_ok()
}

/// Core JSON-over-FFI entrypoint.
///
/// request_json: JSON-encoded BleGuardRequest (FfiIntent + optional FfiLinkParams)
/// Returns: JSON-encoded BleGuardResponse.
#[no_mangle]
pub extern "C" fn evaluate_ble_guard_json(request_json: *const c_char) -> *mut c_char {
    let result = (|| -> Result<BleGuardResponse, String> {
        let guard_cell = GUARD
            .get()
            .ok_or_else(|| "Guard not initialized".to_string())?;

        let request_str = str_from_ptr(request_json)
            .map_err(|_| "Invalid UTF-8 in request_json".to_string())?;

        let req: BleGuardRequest =
            serde_json::from_str(request_str).map_err(|e| format!("Request JSON parse error: {e}"))?;

        let mut guard = guard_cell
            .lock()
            .map_err(|_| "Guard mutex poisoned".to_string())?;

        let (intent, link) = convert_request(req)?;
        let decision = match (&intent, link.as_ref()) {
            (BleIntent::Scan { .. }, _) => guard.guard_scan(),
            (BleIntent::Connect { .. }, Some(lp)) => guard.guard_connect(&intent, lp),
            (BleIntent::SubscribeCharacteristic { .. }, _) => guard.guard_subscribe(&intent),
            (BleIntent::WriteCharacteristic { .. }, Some(lp)) => guard.guard_write(&intent, lp),
            _ => BleGuardDecision::Rejected {
                reason: "Missing link params for connect/write".into(),
            },
        };

        let response = match decision {
            BleGuardDecision::Allowed => BleGuardResponse {
                decision: "allowed".into(),
                reason: None,
            },
            BleGuardDecision::Rejected { reason } => BleGuardResponse {
                decision: "rejected".into(),
                reason: Some(reason),
            },
        };

        Ok(response)
    })();

    let json = match result {
        Ok(resp) => match serde_json::to_string(&resp) {
            Ok(s) => s,
            Err(_) => {
                "{\"decision\":\"rejected\",\"reason\":\"serialization error\"}".to_string()
            }
        },
        Err(e) => match serde_json::to_string(&BleGuardResponse {
            decision: "rejected".into(),
            reason: Some(e),
        }) {
            Ok(s) => s,
            Err(_) => {
                "{\"decision\":\"rejected\",\"reason\":\"serialization error\"}".to_string()
            }
        },
    };

    let cstring = match CString::new(json) {
        Ok(s) => s,
        Err(_) => {
            let fallback = "{\"decision\":\"rejected\",\"reason\":\"ffi CString error\"}";
            match CString::new(fallback) {
                Ok(s) => s,
                Err(_) => return core::ptr::null_mut(),
            }
        }
    };

    cstring.into_raw()
}

fn convert_request(req: BleGuardRequest) -> Result<(BleIntent, Option<BleLinkParams>), String> {
    let intent = match req.intent {
        FfiIntent::Scan => BleIntent::Scan { class_id: None },
        FfiIntent::Connect {
            class_id,
            device_id,
        } => BleIntent::Connect {
            class_id,
            device_id,
        },
        FfiIntent::SubscribeCharacteristic {
            class_id,
            device_id,
            service_uuid,
            characteristic_uuid,
        } => BleIntent::SubscribeCharacteristic {
            class_id,
            device_id,
            service_uuid,
            characteristic_uuid,
        },
        FfiIntent::WriteCharacteristic {
            class_id,
            device_id,
            service_uuid,
            characteristic_uuid,
            payload_len,
        } => BleIntent::WriteCharacteristic {
            class_id,
            device_id,
            service_uuid,
            characteristic_uuid,
            payload_len,
        },
    };

    let link = req.link.map(|l| BleLinkParams {
        phy: parse_phy(&l.phy),
        encrypted: l.encrypted,
        mic_present: l.mic_present,
        bonded: l.bonded,
        conn_interval_ms: l.conn_interval_ms,
        max_pdu_bytes: l.max_pdu_bytes,
        cte_present: l.cte_present,
    });

    Ok((intent, link))
}

fn parse_phy(s: &str) -> ble_model::BlePhy {
    match s {
        "LE2M" => ble_model::BlePhy::Le2M,
        "LECodedS2" | "LECODEDS2" => ble_model::BlePhy::LeCodedS2,
        "LECodedS8" | "LECODEDS8" => ble_model::BlePhy::LeCodedS8,
        _ => ble_model::BlePhy::Le1M,
    }
}
