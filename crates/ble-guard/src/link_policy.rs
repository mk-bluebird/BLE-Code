#![forbid(unsafe_code)]

use ble_governance::DeviceClassPolicy;
use ble_model::{BleLinkParams, BlePhy};

/// Check link parameters against a given DeviceClassPolicy.
///
/// This enforces:
/// - PHY within min_phy..=max_phy
/// - encryption / MIC / bonding flags
/// - conn_interval_ms <= max_conn_interval_ms
/// - max_pdu_bytes <= max_pdu_bytes
/// - cte_present allowed only when allow_cte is true
pub fn check_link_against_class(
    policy: &DeviceClassPolicy,
    link: &BleLinkParams,
) -> Result<(), String> {
    // PHY bounds: convert strings in policy to enum for comparison.
    let min_phy = parse_phy(&policy.min_phy)
        .ok_or_else(|| format!("Invalid min_phy in policy: {}", policy.min_phy))?;
    let max_phy = parse_phy(&policy.max_phy)
        .ok_or_else(|| format!("Invalid max_phy in policy: {}", policy.max_phy))?;

    if !phy_in_range(link.phy, min_phy, max_phy) {
        return Err(format!(
            "PHY {:?} out of allowed range [{:?}, {:?}]",
            link.phy, min_phy, max_phy
        ));
    }

    if policy.require_encryption && !link.encrypted {
        return Err("Encryption required by policy".into());
    }

    if policy.require_mic && !link.mic_present {
        return Err("MIC required by policy".into());
    }

    if policy.require_bonding && !link.bonded {
        return Err("Bonding required by policy".into());
    }

    if link.conn_interval_ms > policy.max_conn_interval_ms {
        return Err(format!(
            "Connection interval {}ms exceeds maximum {}ms",
            link.conn_interval_ms, policy.max_conn_interval_ms
        ));
    }

    if link.max_pdu_bytes > policy.max_pdu_bytes {
        return Err(format!(
            "PDU size {}B exceeds maximum {}B",
            link.max_pdu_bytes, policy.max_pdu_bytes
        ));
    }

    if link.cte_present && !policy.allow_cte {
        return Err("CTE not allowed for this device class".into());
    }

    Ok(())
}

fn parse_phy(s: &str) -> Option<BlePhy> {
    match s {
        "LE1M" => Some(BlePhy::Le1M),
        "LE2M" => Some(BlePhy::Le2M),
        "LECodedS2" | "LECODEDS2" => Some(BlePhy::LeCodedS2),
        "LECodedS8" | "LECODEDS8" => Some(BlePhy::LeCodedS8),
        _ => None,
    }
}

fn phy_in_range(phy: BlePhy, min: BlePhy, max: BlePhy) -> bool {
    // Order: LE1M < LE2M < LECodedS2 < LECodedS8
    fn rank(p: BlePhy) -> u8 {
        match p {
            BlePhy::Le1M => 0,
            BlePhy::Le2M => 1,
            BlePhy::LeCodedS2 => 2,
            BlePhy::LeCodedS8 => 3,
        }
    }
    let r = rank(phy);
    let r_min = rank(min);
    let r_max = rank(max);
    r >= r_min && r <= r_max
}
