mod common;

use api::infra::pg::profile_repo::PgProfileRepository;
use api::profile::repository::ProfileRepository;
use api::profile::models::{Profile, UserPreference, MedicalSafetyProfile, UserGoal};

#[tokio::test]
async fn test_profile_crud_and_goal_override() {
    let (_state, db, _) = common::setup_test_state(false, false).await;
    let (user_id, _) = common::setup_test_user(&db).await;

    let repo = PgProfileRepository::new(db.clone());

    // 1. Profile CRUD
    let profile = Profile {
        user_id,
        full_name: Some("John Doe".to_string()),
        avatar_url: Some("http://avatar.url".to_string()),
        dob: Some(chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        gender: Some("male".to_string()),
        timezone: "UTC".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    repo.upsert_profile(profile.clone()).await.unwrap();

    let fetched = repo.get_profile(user_id).await.unwrap().unwrap();
    assert_eq!(fetched.full_name, Some("John Doe".to_string()));
    assert_eq!(fetched.gender, Some("male".to_string()));

    // Update full name
    let mut updated_profile = fetched;
    updated_profile.full_name = Some("Jane Doe".to_string());
    repo.upsert_profile(updated_profile).await.unwrap();

    let fetched_updated = repo.get_profile(user_id).await.unwrap().unwrap();
    assert_eq!(fetched_updated.full_name, Some("Jane Doe".to_string()));

    // 2. Medical Safety Profile
    let safety = MedicalSafetyProfile {
        user_id,
        allergies: vec!["peanut".to_string(), "soy".to_string()],
        medical_conditions: vec!["diabetes".to_string()],
        is_pregnant: false,
        is_lactating: false,
        updated_at: chrono::Utc::now(),
    };

    repo.upsert_medical_safety(safety).await.unwrap();

    let fetched_safety = repo.get_medical_safety(user_id).await.unwrap().unwrap();
    assert_eq!(fetched_safety.allergies, vec!["peanut", "soy"]);
    assert_eq!(fetched_safety.medical_conditions, vec!["diabetes"]);

    // 3. User Preferences
    let preferences = UserPreference {
        user_id,
        theme: "dark".to_string(),
        language: "en".to_string(),
        measurement_system: "metric".to_string(),
        preferences: serde_json::json!({ "preferred_cuisine": "Italian" }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    repo.upsert_preferences(preferences).await.unwrap();

    let fetched_prefs = repo.get_preferences(user_id).await.unwrap().unwrap();
    assert_eq!(fetched_prefs.preferences["preferred_cuisine"], "Italian");

    // 4. Goal Category Deactivation Overrides
    let goal_id_1 = api::common::id::GoalId::new();
    let goal1 = UserGoal {
        id: goal_id_1,
        user_id,
        category: "nutrition".to_string(),
        target_type: "calories".to_string(),
        target_value: 2000.0,
        unit: "kcal".to_string(),
        config: serde_json::json!({}),
        start_date: chrono::Utc::now().date_naive(),
        target_date: None,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repo.create_goal(goal1).await.unwrap();

    let active_goals = repo.get_active_goals(user_id).await.unwrap();
    assert_eq!(active_goals.len(), 1);
    assert_eq!(active_goals[0].target_value, 2000.0);

    // Deactivate goals in nutrition category (simulating service override)
    repo.deactivate_goal(user_id, "nutrition").await.unwrap();

    let goal_id_2 = api::common::id::GoalId::new();
    let goal2 = UserGoal {
        id: goal_id_2,
        user_id,
        category: "nutrition".to_string(),
        target_type: "calories".to_string(),
        target_value: 2500.0,
        unit: "kcal".to_string(),
        config: serde_json::json!({}),
        start_date: chrono::Utc::now().date_naive(),
        target_date: None,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repo.create_goal(goal2).await.unwrap();

    let active_goals_after = repo.get_active_goals(user_id).await.unwrap();
    assert_eq!(active_goals_after.len(), 1);
    assert_eq!(active_goals_after[0].target_value, 2500.0);
}
