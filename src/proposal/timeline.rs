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
    Review(ReviewStatusV1),
    Approved(ReviewStatusV1),
    Rejected(ReviewStatusV1),
    ApprovedConditionally(ReviewStatusV1),
    PaymentProcessing(PaymentProcessingStatusV1),
    Funded(FundedStatusV1),
    Cancelled(ReviewStatusV1),
}

fn convert_review_status_to_v1(review_status: ReviewStatus, kyc_verified: bool) -> ReviewStatusV1 {
    ReviewStatusV1 {
        sponsor_requested_review: review_status.sponsor_requested_review,
        reviewer_completed_attestation: review_status.reviewer_completed_attestation,
        kyc_verified_review: kyc_verified,
    }
}

impl From<PaymentProcessingStatus> for PaymentProcessingStatusV1 {
    fn from(value: PaymentProcessingStatus) -> Self {
        PaymentProcessingStatusV1 {
            review_status: convert_review_status_to_v1(value.review_status, value.kyc_verified),
            kyc_verified: value.kyc_verified,
            test_transaction_sent: value.test_transaction_sent,
            request_for_trustees_created: value.request_for_trustees_created,
        }
    }
}

impl From<FundedStatus> for FundedStatusV1 {
    fn from(value: FundedStatus) -> Self {
        FundedStatusV1 {
            payment_processing_status: value.payment_processing_status.into(),
            trustees_released_payment: value.trustees_released_payment,
            payouts: value.payouts,
        }
    }
}

impl From<TimelineStatus> for TimelineStatusV1 {
    fn from(value: TimelineStatus) -> Self {
        match value {
            TimelineStatus::Draft => TimelineStatusV1::Draft,
            TimelineStatus::Review(review_status) => TimelineStatusV1::Review(convert_review_status_to_v1(review_status, false)),
            TimelineStatus::Approved(review_status) => TimelineStatusV1::Approved(convert_review_status_to_v1(review_status, false)),
            TimelineStatus::Rejected(review_status) => TimelineStatusV1::Rejected(convert_review_status_to_v1(review_status, false)),
            TimelineStatus::ApprovedConditionally(review_status) => {
                TimelineStatusV1::ApprovedConditionally(convert_review_status_to_v1(review_status, false))
            }
            TimelineStatus::PaymentProcessing(payment_processing_status) => {
                TimelineStatusV1::PaymentProcessing(payment_processing_status.into())
            }
            TimelineStatus::Funded(funded_status) => TimelineStatusV1::Funded(funded_status.into()),
            TimelineStatus::Cancelled(review_status) => TimelineStatusV1::Cancelled(convert_review_status_to_v1(review_status, false)),
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

    pub fn was_approved(&self) -> bool {
        match self {
            TimelineStatusV1::Approved(..) => true,
            TimelineStatusV1::ApprovedConditionally(..) => true,
            TimelineStatusV1::PaymentProcessing(..) => true,
            TimelineStatusV1::Funded(..) => true,
            _ => false,
            
        }
    }

    pub fn get_review_status(&self) -> &ReviewStatusV1 {
        match self {
            TimelineStatusV1::Review(review_status)
            | TimelineStatusV1::Approved(review_status)
            | TimelineStatusV1::Rejected(review_status)
            | TimelineStatusV1::ApprovedConditionally(review_status)
            | TimelineStatusV1::Cancelled(review_status) => review_status.into(),
            TimelineStatusV1::PaymentProcessing(payment_processing_status) => {
                &payment_processing_status.review_status
            },
            TimelineStatusV1::Funded(funded_status) => {
                &funded_status.payment_processing_status.review_status
            },
            TimelineStatusV1::Draft => &ReviewStatusV1 {
                sponsor_requested_review: false,
                reviewer_completed_attestation: false,
                kyc_verified_review: false,
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
pub struct ReviewStatusV1 {
    sponsor_requested_review: bool,
    reviewer_completed_attestation: bool,
    kyc_verified_review: bool,
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
    review_status: ReviewStatusV1,
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct FundedStatusV1 {
    #[serde(flatten)]
    payment_processing_status: PaymentProcessingStatusV1,
    trustees_released_payment: bool,
    payouts: Vec<String>,
}
