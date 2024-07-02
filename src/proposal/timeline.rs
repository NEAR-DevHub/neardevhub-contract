use near_sdk::near;

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimelineStatus {
    Draft,
    Review(ReviewStatus),
    Approved(ReviewStatus),
    Rejected(ReviewStatus),
    ApprovedConditionally(ReviewStatus),
    PaymentProcessing(PaymentProcessingStatus),
    Funded(FundedStatus),
    Cancelled(ReviewStatus),
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

    pub fn is_review(&self) -> bool {
        matches!(self, TimelineStatus::Review(..))
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, TimelineStatus::Cancelled(..))
    }

    pub fn can_be_cancelled(&self) -> bool {
        match self {
            TimelineStatus::Draft => true,
            TimelineStatus::Review(..) => true,
            _ => false,
        }
    }

    pub fn was_approved(&self) -> bool {
        match self {
            TimelineStatus::Approved(..) => true,
            TimelineStatus::ApprovedConditionally(..) => true,
            TimelineStatus::PaymentProcessing(..) => true,
            TimelineStatus::Funded(..) => true,
            _ => false,
            
        }
    }

    pub fn get_review_status(&self) -> &ReviewStatus {
        match self {
            TimelineStatus::Review(review_status)
            | TimelineStatus::Approved(review_status)
            | TimelineStatus::Rejected(review_status)
            | TimelineStatus::ApprovedConditionally(review_status)
            | TimelineStatus::Cancelled(review_status) => review_status,
            TimelineStatus::PaymentProcessing(payment_processing_status) => {
                &payment_processing_status.review_status
            },
            TimelineStatus::Funded(funded_status) => {
                &funded_status.payment_processing_status.review_status
            },
            TimelineStatus::Draft => &ReviewStatus {
                sponsor_requested_review: false,
                reviewer_completed_attestation: false,
            },
        }
    }
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct ReviewStatus {
    sponsor_requested_review: bool,
    reviewer_completed_attestation: bool,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct PaymentProcessingStatus {
    #[serde(flatten)]
    review_status: ReviewStatus,
    kyc_verified: bool,
    test_transaction_sent: bool,
    request_for_trustees_created: bool,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct FundedStatus {
    #[serde(flatten)]
    payment_processing_status: PaymentProcessingStatus,
    trustees_released_payment: bool,
    payouts: Vec<String>,
}
