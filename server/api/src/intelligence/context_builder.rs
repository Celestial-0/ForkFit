use crate::app::AppState;
use crate::common::AppResult;
use crate::common::id::{UserId, ChatThreadId};
use crate::gateway::grpc_client::intelligence::{UserContext, ChatMessageHistory};

// Repositories
use crate::infra::pg::profile_repo::PgProfileRepository;
use crate::infra::pg::intelligence_repo::PgIntelligenceRepository;
use crate::infra::pg::chat_repo::PgChatRepository;

use crate::profile::repository::ProfileRepository;
use crate::intelligence::repository::IntelligenceRepository;
use crate::chat::repository::ChatRepository;

pub async fn build_user_context(state: &AppState, user_id: UserId) -> AppResult<UserContext> {
    let profile_repo = PgProfileRepository::new(state.db.clone());
    let intel_repo = PgIntelligenceRepository::new(state.db.clone());

    // Run parallel database queries using tokio::try_join!
    let (preferences, safety, goals, memories) = tokio::try_join!(
        profile_repo.get_preferences(user_id),
        profile_repo.get_medical_safety(user_id),
        profile_repo.get_active_goals(user_id),
        intel_repo.get_active_memories(user_id),
    )?;

    // Parse cuisine, preferred/avoided foods from preferences or memories
    let mut preferred_cuisine = "Standard".to_string();
    let mut preferred_foods = Vec::new();
    let mut avoided_foods = Vec::new();

    if let Some(prefs) = preferences {
        if let Some(cuisine) = prefs.preferences.get("preferred_cuisine").and_then(|c| c.as_str()) {
            preferred_cuisine = cuisine.to_string();
        }
        if let Some(arr) = prefs.preferences.get("preferred_foods").and_then(|a| a.as_array()) {
            preferred_foods = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
        }
        if let Some(arr) = prefs.preferences.get("avoided_foods").and_then(|a| a.as_array()) {
            avoided_foods = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
        }
    }

    // Add preferences extracted from long-term memory content
    for memory in memories {
        if memory.memory_type == "preference" {
            preferred_foods.push(memory.content);
        } else if memory.memory_type == "restriction" {
            avoided_foods.push(memory.content);
        }
    }

    // Setup nutrition target defaults
    let mut daily_calorie_target = 2000.0;
    let mut target_protein_g = 150.0;
    let mut target_carbs_g = 200.0;
    let mut target_fat_g = 70.0;
    let mut budget_limit = 500.0; // Default budget limit (INR)

    for goal in goals {
        let t_type = goal.target_type.to_lowercase();
        if t_type == "calories" || t_type == "calorie" {
            daily_calorie_target = goal.target_value;
        } else if t_type == "protein" {
            target_protein_g = goal.target_value;
        } else if t_type == "carbs" || t_type == "carbohydrates" {
            target_carbs_g = goal.target_value;
        } else if t_type == "fat" || t_type == "fats" {
            target_fat_g = goal.target_value;
        } else if t_type == "budget_limit" || t_type == "budget" {
            budget_limit = goal.target_value;
        }
    }

    let (allergies, medical_conditions) = if let Some(s) = safety {
        (s.allergies, s.medical_conditions)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(UserContext {
        user_id: user_id.to_string(),
        allergies,
        medical_conditions,
        preferred_foods,
        avoided_foods,
        daily_calorie_target,
        target_protein_g,
        target_carbs_g,
        target_fat_g,
        budget_limit,
        preferred_cuisine,
    })
}

pub async fn build_chat_history(state: &AppState, thread_id: ChatThreadId, limit: u64) -> AppResult<Vec<ChatMessageHistory>> {
    let chat_repo = PgChatRepository::new(state.db.clone());
    
    // Fetch last messages (usually page 1 with limit)
    let (messages, _) = chat_repo.list_messages(thread_id, 1, limit).await?;

    let mut history = Vec::new();
    for msg in messages {
        let role = msg.sender_role.as_str().to_string();
        
        let seconds = msg.created_at.timestamp();
        let nanos = msg.created_at.timestamp_subsec_nanos() as i32;
        let sent_at = prost_types::Timestamp {
            seconds,
            nanos,
        };

        history.push(ChatMessageHistory {
            role,
            content: msg.content,
            sent_at: Some(sent_at),
        });
    }

    Ok(history)
}
