use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{JudgeDeviceId, JudgeId};

/// A short-lived code a judge types into their iPad (or scans as a QR
/// code) to pair it with their `Judge` record. Generic infrastructure,
/// not gymnastics-specific -- any sport's judges pair the same way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingCode {
    pub code: String,
    pub judge_id: JudgeId,
    pub expires_at: DateTime<Utc>,
    pub consumed: bool,
}

/// A paired device (an iPad) authenticated to submit scores as a
/// specific judge. `token_hash` never leaves the backend -- the
/// plaintext token is handed to the device once, at pairing time, and
/// only its hash is ever stored or serialized from here on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeDevice {
    pub id: JudgeDeviceId,
    pub judge_id: JudgeId,
    pub device_name: String,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub paired_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}
