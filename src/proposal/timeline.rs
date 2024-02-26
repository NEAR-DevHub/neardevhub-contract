use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::NearSchema;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde", tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
#[borsh(crate = "near_sdk::borsh")]
pub enum TimelineStatus {
    Draft,
    Review(ReviewStatus),
    Approved(ReviewStatus),
    Rejected(ReviewStatus),
    ApprovedConditionally(ReviewStatus),
    PaymentProcessing(PaymentProcessingStatus),
    Funded(FundedStatus),
}

impl TimelineStatus {
    pub fn is_draft(&self) -> bool {
        matches!(self, TimelineStatus::Draft)
    }

    pub fn is_empty_review(&self) -> bool {
        match self {
            TimelineStatus::Review(review_status) => {
                !review_status.sponsor_requested_review
                    && !review_status.reviewer_completed_attestation
            }
            _ => false,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct ReviewStatus {
    sponsor_requested_review: bool,
    reviewer_completed_attestation: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct PaymentProcessingStatus {
    #[serde(flatten)]
    review_status: ReviewStatus,
    kyc_verified: bool,
    test_transaction_sent: bool,
    request_for_trustees_created: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct FundedStatus {
    #[serde(flatten)]
    payment_processing_status: PaymentProcessingStatus,
    trustees_released_payment: bool,
}
