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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "timeline_version")]
pub enum VersionedTimelineStatus {
    V1(TimelineStatusV1),
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimelineStatusV1 {
    Draft,
    Review(ReviewStatus),
    Approved(ReviewStatus),
    Rejected(ReviewStatus),
    ApprovedConditionally(ReviewStatus),
    PaymentProcessing(PaymentProcessingStatusV1),
    Funded(FundedStatus),
    Cancelled(ReviewStatus),
}

impl From<TimelineStatus> for TimelineStatusV1 {
    fn from(value: TimelineStatus) -> Self {
        match value {
            TimelineStatus::Draft => TimelineStatusV1::Draft,
            TimelineStatus::Review(review_status) => TimelineStatusV1::Review(review_status),
            TimelineStatus::Approved(review_status) => TimelineStatusV1::Approved(review_status),
            TimelineStatus::Rejected(review_status) => TimelineStatusV1::Rejected(review_status),
            TimelineStatus::ApprovedConditionally(review_status) => {
                TimelineStatusV1::ApprovedConditionally(review_status)
            }
            TimelineStatus::PaymentProcessing(payment_processing_status) => {
                TimelineStatusV1::PaymentProcessing(payment_processing_status.into())
            }
            TimelineStatus::Funded(funded_status) => TimelineStatusV1::Funded(funded_status),
            TimelineStatus::Cancelled(review_status) => TimelineStatusV1::Cancelled(review_status),
        }
    }
}

impl VersionedTimelineStatus {
    pub fn latest_version(self) -> TimelineStatusV1 {
        self.into()
    }
}

impl From<VersionedTimelineStatus> for TimelineStatusV1 {
    fn from(value: VersionedTimelineStatus) -> Self {
        match value {
            VersionedTimelineStatus::V1(v1) => v1,
        }
    }
}

impl From<TimelineStatusV1> for VersionedTimelineStatus {
    fn from(value: TimelineStatusV1) -> Self {
        VersionedTimelineStatus::V1(value)
    }
}

impl From<TimelineStatus> for VersionedTimelineStatus {
    fn from(value: TimelineStatus) -> Self {
        VersionedTimelineStatus::V1(value.into())
    }
}

impl TimelineStatusV1 {
    pub fn is_draft(&self) -> bool {
        matches!(self, TimelineStatusV1::Draft)
    }

    pub fn is_empty_review(&self) -> bool {
        match self {
            TimelineStatusV1::Review(review_status) => {
                !review_status.sponsor_requested_review
                    && !review_status.reviewer_completed_attestation
            }
            _ => false,
        }
    }

    pub fn is_review(&self) -> bool {
        matches!(self, TimelineStatusV1::Review(..))
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, TimelineStatusV1::Cancelled(..))
    }

    pub fn can_be_cancelled(&self) -> bool {
        match self {
            TimelineStatusV1::Draft => true,
            TimelineStatusV1::Review(..) => true,
            _ => false,
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
pub struct PaymentProcessingStatusV1 {
    #[serde(flatten)]
    review_status: ReviewStatus,
    kyc_verified: bool,
    test_transaction_sent: bool,
    request_for_trustees_created: bool,
    approved_conditionally: bool,
}

impl From<PaymentProcessingStatus> for PaymentProcessingStatusV1 {
    fn from(payment_processing_status: PaymentProcessingStatus) -> Self {
        PaymentProcessingStatusV1 {
            review_status: payment_processing_status.review_status,
            kyc_verified: payment_processing_status.kyc_verified,
            test_transaction_sent: payment_processing_status.test_transaction_sent,
            request_for_trustees_created: payment_processing_status.request_for_trustees_created,
            approved_conditionally: false,
        }
    }
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct FundedStatus {
    #[serde(flatten)]
    payment_processing_status: PaymentProcessingStatus,
    trustees_released_payment: bool,
    payouts: Vec<String>,
}
