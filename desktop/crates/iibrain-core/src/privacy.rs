//! The privacy gate: the one place that says whether something may go
//! to a Model (ADR 0004). What is Private reaches only a Model the
//! person declared local.

use crate::model::Model;

/// Whether something that is Private or not may be sent to `model`.
pub fn may_send(private: bool, model: &Model) -> bool {
    !private || model.local
}
