// File: crates/corridor_nanoswarm_telemetry/src/payload.rs
// License: MIT OR Apache-2.0

use core::fmt;

/// Message type for nanoswarm telemetry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    EmfAcousticSummary = 0x01,
    Control = 0x02,
}

impl MsgType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(MsgType::EmfAcousticSummary),
            0x02 => Some(MsgType::Control),
            _ => None,
        }
    }
}

/// Compact histogram representation.
/// Each bin is a 4-bit value; two bins per byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Histogram {
    /// Bin count; must be <= 32 to fit into 16 bytes (4-bit bins).
    pub bins: u8,
    /// Packed 4-bit bins; length must be >= ceil(bins / 2).
    pub data: heapless::Vec<u8, 16>,
}

impl Histogram {
    pub fn new() -> Self {
        Histogram {
            bins: 0,
            data: heapless::Vec::new(),
        }
    }

    pub fn from_bins(values: &[u8]) -> Option<Self> {
        if values.len() > 32 {
            return None;
        }
        let mut hist = Histogram::new();
        hist.bins = values.len() as u8;

        let mut i = 0usize;
        while i < values.len() {
            let high = values[i] & 0x0F;
            let mut byte = high << 4;
            if i + 1 < values.len() {
                let low = values[i + 1] & 0x0F;
                byte |= low;
            }
            if hist.data.push(byte).is_err() {
                return None;
            }
            i += 2;
        }
        Some(hist)
    }

    pub fn decode_bins(&self, output: &mut [u8]) -> usize {
        let mut count = 0usize;
        for &byte in self.data.iter() {
            if count >= self.bins as usize {
                break;
            }
            output[count] = (byte >> 4) & 0x0F;
            count += 1;
            if count >= self.bins as usize {
                break;
            }
            output[count] = byte & 0x0F;
            count += 1;
        }
        count
    }
}

/// Nanoswarm telemetry payload (BLE / LoRaWAN shared grammar).
///
/// Layout (little-endian, total length <= 31 bytes for BLE):
/// - msg_type: 1 byte
/// - slice_id: 2 bytes
/// - node_id: 2 bytes
/// - version: 1 byte
/// - hist_emf: variable (Histogram)
/// - hist_acoustic: variable (Histogram)
/// - v_resid_q: 1 byte (quantized residual, optional)
/// - checksum: 1 byte
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NanoswarmPayload {
    pub msg_type: MsgType,
    pub slice_id: u16,
    pub node_id: u16,
    pub version: u8,
    pub hist_emf: Histogram,
    pub hist_acoustic: Histogram,
    /// Quantized residual or RF risk coordinate (0-255).
    pub v_resid_q: Option<u8>,
    pub checksum: u8,
}

impl NanoswarmPayload {
    pub fn compute_checksum(bytes: &[u8]) -> u8 {
        let mut sum: u8 = 0;
        for &b in bytes {
            sum = sum.wrapping_add(b);
        }
        !sum
    }

    pub fn encode(&self, buffer: &mut [u8]) -> Option<usize> {
        // encode into buffer; return length
        let mut idx = 0usize;

        if buffer.len() < 8 {
            return None;
        }

        buffer[idx] = self.msg_type as u8;
        idx += 1;

        buffer[idx] = (self.slice_id & 0xFF) as u8;
        buffer[idx + 1] = (self.slice_id >> 8) as u8;
        idx += 2;

        buffer[idx] = (self.node_id & 0xFF) as u8;
        buffer[idx + 1] = (self.node_id >> 8) as u8;
        idx += 2;

        buffer[idx] = self.version;
        idx += 1;

        // encode hist_emf
        buffer[idx] = self.hist_emf.bins;
        idx += 1;
        for &b in self.hist_emf.data.iter() {
            if idx >= buffer.len() {
                return None;
            }
            buffer[idx] = b;
            idx += 1;
        }

        // encode hist_acoustic
        buffer[idx] = self.hist_acoustic.bins;
        idx += 1;
        for &b in self.hist_acoustic.data.iter() {
            if idx >= buffer.len() {
                return None;
            }
            buffer[idx] = b;
            idx += 1;
        }

        if let Some(vr) = self.v_resid_q {
            if idx >= buffer.len() {
                return None;
            }
            buffer[idx] = vr;
            idx += 1;
        }

        if idx >= buffer.len() {
            return None;
        }
        let checksum = Self::compute_checksum(&buffer[..idx]);
        buffer[idx] = checksum;
        let total_len = idx + 1;

        Some(total_len)
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        let mut idx = 0usize;

        let msg_type = MsgType::from_u8(bytes[idx])?;
        idx += 1;

        let slice_id = u16::from_le_bytes([bytes[idx], bytes[idx + 1]]);
        idx += 2;

        let node_id = u16::from_le_bytes([bytes[idx], bytes[idx + 1]]);
        idx += 2;

        let version = bytes[idx];
        idx += 1;

        // hist_emf
        let emf_bins = bytes[idx];
        idx += 1;
        let emf_bytes_len = ((emf_bins as usize + 1) / 2).min(bytes.len().saturating_sub(idx));
        let mut emf_data = heapless::Vec::<u8, 16>::new();
        for i in 0..emf_bytes_len {
            if emf_data.push(bytes[idx + i]).is_err() {
                return None;
            }
        }
        idx += emf_bytes_len;
        let hist_emf = Histogram {
            bins: emf_bins,
            data: emf_data,
        };

        // hist_acoustic
        if idx >= bytes.len() {
            return None;
        }
        let ac_bins = bytes[idx];
        idx += 1;
        let ac_bytes_len = ((ac_bins as usize + 1) / 2).min(bytes.len().saturating_sub(idx));
        let mut ac_data = heapless::Vec::<u8, 16>::new();
        for i in 0..ac_bytes_len {
            if ac_data.push(bytes[idx + i]).is_err() {
                return None;
            }
        }
        idx += ac_bytes_len;
        let hist_acoustic = Histogram {
            bins: ac_bins,
            data: ac_data,
        };

        if bytes.len() < idx + 1 {
            return None;
        }

        let remaining = bytes.len() - idx;
        let (v_resid_q, checksum_idx) = if remaining == 1 {
            (None, idx)
        } else {
            (Some(bytes[idx]), idx + 1)
        };

        if checksum_idx >= bytes.len() {
            return None;
        }

        let checksum = bytes[bytes.len() - 1];
        let computed = Self::compute_checksum(&bytes[..bytes.len() - 1]);
        if checksum != computed {
            return None;
        }

        Some(NanoswarmPayload {
            msg_type,
            slice_id,
            node_id,
            version,
            hist_emf,
            hist_acoustic,
            v_resid_q,
            checksum,
        })
    }
}

impl fmt::Display for NanoswarmPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MsgType={:?}, slice_id={}, node_id={}, version={}, v_resid_q={:?}",
            self.msg_type, self.slice_id, self.node_id, self.version, self.v_resid_q
        )
    }
}
