use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::common::id::{UserId, GoalId, BiometricLogId, WorkoutLogId};
use super::models::{UserPreference, MedicalSafetyProfile, BiometricLog, WorkoutLog, UserGoal};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<NaiveDate>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    pub user_id: UserId,
    pub email: String,
    pub email_verified: bool,
    pub status: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<NaiveDate>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMedicalSafetyRequest {
    pub allergies: Vec<String>,
    pub medical_conditions: Vec<String>,
    pub is_pregnant: bool,
    pub is_lactating: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalSafetyResponse {
    pub user_id: UserId,
    pub allergies: Vec<String>,
    pub medical_conditions: Vec<String>,
    pub is_pregnant: bool,
    pub is_lactating: bool,
    pub updated_at: DateTime<Utc>,
}

impl From<MedicalSafetyProfile> for MedicalSafetyResponse {
    fn from(m: MedicalSafetyProfile) -> Self {
        Self {
            user_id: m.user_id,
            allergies: m.allergies,
            medical_conditions: m.medical_conditions,
            is_pregnant: m.is_pregnant,
            is_lactating: m.is_lactating,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePreferenceRequest {
    pub theme: String,
    pub language: String,
    pub measurement_system: String,
    pub preferences: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferenceResponse {
    pub user_id: UserId,
    pub theme: String,
    pub language: String,
    pub measurement_system: String,
    pub preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserPreference> for UserPreferenceResponse {
    fn from(up: UserPreference) -> Self {
        Self {
            user_id: up.user_id,
            theme: up.theme,
            language: up.language,
            measurement_system: up.measurement_system,
            preferences: up.preferences,
            created_at: up.created_at,
            updated_at: up.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBiometricRequest {
    pub metric_type: String,
    pub value: f64,
    pub notes: Option<String>,
    pub logged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricResponse {
    pub id: BiometricLogId,
    pub user_id: UserId,
    pub logged_at: DateTime<Utc>,
    pub metric_type: String,
    pub value: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<BiometricLog> for BiometricResponse {
    fn from(b: BiometricLog) -> Self {
        Self {
            id: b.id,
            user_id: b.user_id,
            logged_at: b.logged_at,
            metric_type: b.metric_type,
            value: b.value,
            notes: b.notes,
            created_at: b.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkoutRequest {
    pub activity_name: String,
    pub duration_minutes: i32,
    pub calories_burned: f64,
    pub notes: Option<String>,
    pub logged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutResponse {
    pub id: WorkoutLogId,
    pub user_id: UserId,
    pub logged_at: DateTime<Utc>,
    pub activity_name: String,
    pub duration_minutes: i32,
    pub calories_burned: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<WorkoutLog> for WorkoutResponse {
    fn from(w: WorkoutLog) -> Self {
        Self {
            id: w.id,
            user_id: w.user_id,
            logged_at: w.logged_at,
            activity_name: w.activity_name,
            duration_minutes: w.duration_minutes,
            calories_burned: w.calories_burned,
            notes: w.notes,
            created_at: w.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoalRequest {
    pub category: String,
    pub target_type: String,
    pub target_value: f64,
    pub unit: String,
    pub config: serde_json::Value,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalResponse {
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

impl From<UserGoal> for GoalResponse {
    fn from(g: UserGoal) -> Self {
        Self {
            id: g.id,
            user_id: g.user_id,
            category: g.category,
            target_type: g.target_type,
            target_value: g.target_value,
            unit: g.unit,
            config: g.config,
            start_date: g.start_date,
            target_date: g.target_date,
            is_active: g.is_active,
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}
