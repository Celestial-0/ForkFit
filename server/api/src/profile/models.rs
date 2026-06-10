use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::common::id::{UserId, GoalId, BiometricLogId, WorkoutLogId};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Profile {
    pub user_id: UserId,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<NaiveDate>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub user_id: UserId,
    pub theme: String,
    pub language: String,
    pub measurement_system: String,
    pub preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MedicalSafetyProfile {
    pub user_id: UserId,
    pub allergies: Vec<String>,
    pub medical_conditions: Vec<String>,
    pub is_pregnant: bool,
    pub is_lactating: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricLog {
    pub id: BiometricLogId,
    pub user_id: UserId,
    pub logged_at: DateTime<Utc>,
    pub metric_type: String,
    pub value: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutLog {
    pub id: WorkoutLogId,
    pub user_id: UserId,
    pub logged_at: DateTime<Utc>,
    pub activity_name: String,
    pub duration_minutes: i32,
    pub calories_burned: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGoal {
    pub id: GoalId,
    pub user_id: UserId,
    pub category: String,
    pub target_type: String,
    pub target_value: f64,
    pub unit: String,
    pub config: serde_json::Value,
    pub start_date: NaiveDate,
    pub target_date: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
