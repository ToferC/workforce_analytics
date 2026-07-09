use chrono::{prelude::*, Datelike, Duration};
use async_graphql::*;
use uuid::Uuid;

/// Time bucket granularity for analytics series
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TimeBucket {
    Week,
    Month,
    Quarter,
    Year,
}

impl TimeBucket {
    /// Snap `dt` to the start of its containing bucket
    pub fn bucket_start(&self, dt: NaiveDateTime) -> NaiveDateTime {
        match self {
            TimeBucket::Week => {
                // ISO week starts on Monday
                let days_from_monday = dt.weekday().num_days_from_monday();
                let date = dt.date() - Duration::days(days_from_monday as i64);
                date.and_hms_opt(0, 0, 0).unwrap()
            }
            TimeBucket::Month => {
                NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
            }
            TimeBucket::Quarter => {
                let quarter_month = (dt.month0() / 3) * 3 + 1;
                NaiveDate::from_ymd_opt(dt.year(), quarter_month, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
            }
            TimeBucket::Year => {
                NaiveDate::from_ymd_opt(dt.year(), 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
            }
        }
    }

    /// Advance one bucket from the given bucket start
    pub fn next_bucket_start(&self, start: NaiveDateTime) -> NaiveDateTime {
        match self {
            TimeBucket::Week => start + Duration::weeks(1),
            TimeBucket::Month => {
                let d = start.date();
                if d.month() == 12 {
                    NaiveDate::from_ymd_opt(d.year() + 1, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                } else {
                    NaiveDate::from_ymd_opt(d.year(), d.month() + 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                }
            }
            TimeBucket::Quarter => {
                let d = start.date();
                let next_month = (d.month0() / 3) * 3 + 4;
                if next_month > 12 {
                    NaiveDate::from_ymd_opt(d.year() + 1, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                } else {
                    NaiveDate::from_ymd_opt(d.year(), next_month, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                }
            }
            TimeBucket::Year => {
                NaiveDate::from_ymd_opt(start.year() + 1, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
            }
        }
    }

    /// One nanosecond before the next bucket (inclusive end of this bucket)
    pub fn bucket_end(&self, start: NaiveDateTime) -> NaiveDateTime {
        self.next_bucket_start(start) - Duration::nanoseconds(1)
    }

    /// Dense list of bucket starts covering [from, to]
    pub fn generate_buckets(&self, from: NaiveDateTime, to: NaiveDateTime) -> Vec<NaiveDateTime> {
        let mut buckets = Vec::new();
        let mut current = self.bucket_start(from);
        while current <= to {
            buckets.push(current);
            current = self.next_bucket_start(current);
        }
        buckets
    }
}

/// A single data point in a time series
#[derive(Debug, Clone, SimpleObject)]
pub struct TimeSeriesPoint {
    /// Start of the bucket, ISO 8601
    pub period_start: NaiveDateTime,
    pub bucket: TimeBucket,
    pub value: f64,
}

/// A named time series (e.g. one per SkillDomain)
#[derive(Debug, Clone, SimpleObject)]
pub struct LabeledSeries {
    /// e.g. a SkillDomain value, as a String
    pub key: String,
    pub points: Vec<TimeSeriesPoint>,
}

/// Supply vs demand for a single time bucket
#[derive(Debug, Clone, SimpleObject)]
pub struct SupplyDemandPoint {
    pub period_start: NaiveDateTime,
    pub bucket: TimeBucket,
    /// Count/weight of validated capabilities available as of period end.
    pub supply: f64,
    /// Count/weight of required capability from role Requirements + Work as of period end.
    pub demand: f64,
}

/// Supply/demand time series for one domain
#[derive(Debug, Clone, SimpleObject)]
pub struct SupplyDemandSeries {
    /// SkillDomain value as String.
    pub domain: String,
    pub points: Vec<SupplyDemandPoint>,
}

/// A cell in the team capability heatmap
#[derive(Debug, Clone, SimpleObject)]
pub struct TeamCapabilityCell {
    pub domain: String,
    /// Weighted capability depth (validated; self-identified fallback).
    pub depth: f64,
    /// Distinct people contributing at this domain.
    pub people_count: i32,
}

/// A row in the team capability heatmap
#[derive(Debug, Clone, SimpleObject)]
pub struct TeamCapabilityRow {
    pub team_id: Uuid,
    pub team_name: String,
    pub cells: Vec<TeamCapabilityCell>,
}

/// A row in the org-tier capability heatmap: the team matrix rolled up to a
/// chosen tier level (e.g. tier 2), for organizations where per-team rows are
/// too granular to read.
#[derive(Debug, Clone, SimpleObject)]
pub struct OrgTierCapabilityRow {
    pub org_tier_id: Uuid,
    pub org_tier_name: String,
    pub tier_level: i32,
    pub cells: Vec<TeamCapabilityCell>,
}

/// A detected talent movement event
#[derive(Debug, Clone, SimpleObject)]
pub struct TalentMovement {
    pub person_id: Uuid,
    pub at: NaiveDateTime,
    pub from_team_id: Option<Uuid>,
    pub to_team_id: Option<Uuid>,
    pub from_level: Option<String>,
    pub to_level: Option<String>,
    /// PROMOTION | LATERAL | INFLOW | OUTFLOW
    pub kind: String,
}
