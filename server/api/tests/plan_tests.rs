mod common;

use api::infra::pg::plan_repo::PgPlanRepository;
use api::infra::pg::recipe_repo::PgRecipeRepository;
use api::recipe::service::RecipeService;
use api::recipe::types::{CreateIngredientRequest, CreateRecipeRequest, RecipeIngredientInput};
use api::plan::service::PlanService;
use api::plan::types::{CreateMealPlanRequest, MealPlanItemInput};

#[tokio::test]
async fn test_active_plan_deactivation() {
    let (_state, db, _) = common::setup_test_state(false, false).await;
    let (user_id, _) = common::setup_test_user(&db).await;

    let recipe_repo = PgRecipeRepository::new(db.clone());
    let recipe_service = RecipeService::new(recipe_repo);
    let plan_repo = PgPlanRepository::new(db.clone());
    let plan_service = PlanService::new(plan_repo, recipe_service.clone());

    // 1. Create a dummy ingredient and recipe
    let ing_req = CreateIngredientRequest {
        name: "Egg".to_string(),
        description: Some("Whole egg".to_string()),
        calories_per_100g: 143.0,
        protein_per_100g: 13.0,
        carbs_per_100g: 1.1,
        fat_per_100g: 11.0,
        fiber_per_100g: Some(0.0),
        sodium_mg_per_100g: Some(124.0),
        micronutrients: Some(serde_json::json!({})),
        estimated_cost_per_100g: Some(2.0),
        price_currency: Some("INR".to_string()),
        barcode: None,
    };
    let ing = recipe_service.create_ingredient(ing_req).await.unwrap();

    let recipe_req = CreateRecipeRequest {
        parent_recipe_id: None,
        title: "Boiled Eggs".to_string(),
        description: Some("Simple boiled eggs".to_string()),
        instructions: vec!["Boil water".to_string(), "Add eggs for 6 mins".to_string()],
        prep_time_minutes: Some(1),
        cook_time_minutes: Some(6),
        servings: 1.0,
        cuisine: Some("Universal".to_string()),
        dietary_tags: vec![],
        is_public: true,
        ingredients: vec![RecipeIngredientInput {
            ingredient_id: ing.id,
            quantity: 100.0,
            unit: "grams".to_string(),
            grams_equivalent: 100.0,
            notes: None,
        }],
    };
    let recipe = recipe_service.create_recipe(user_id, recipe_req).await.unwrap();

    // 2. Create first meal plan as active
    let plan_req1 = CreateMealPlanRequest {
        name: Some("Bulk Week 1".to_string()),
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 16).unwrap(),
        is_active: true,
        items: vec![
            MealPlanItemInput {
                planned_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 11).unwrap(),
                meal_type: "lunch".to_string(),
                recipe_id: Some(recipe.recipe.id),
                ingredient_id: None,
                custom_food_name: None,
                servings: 1.0,
            },
            MealPlanItemInput {
                planned_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
                meal_type: "breakfast".to_string(),
                recipe_id: Some(recipe.recipe.id),
                ingredient_id: None,
                custom_food_name: None,
                servings: 1.0,
            },
        ],
    };

    let plan1 = plan_service.create_meal_plan(user_id, plan_req1).await.unwrap();
    assert!(plan1.plan.is_active);

    // Verify retrieval of active plan returns plan1, sorted by planned_date + meal_type
    let active_plan = plan_service.get_active_meal_plan(user_id).await.unwrap();
    assert_eq!(active_plan.plan.id, plan1.plan.id);
    assert_eq!(active_plan.items.len(), 2);
    // planned_date 2026-06-10 should be first, 2026-06-11 second
    assert_eq!(active_plan.items[0].planned_date, chrono::NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    assert_eq!(active_plan.items[0].meal_type, "breakfast");
    assert_eq!(active_plan.items[1].planned_date, chrono::NaiveDate::from_ymd_opt(2026, 6, 11).unwrap());
    assert_eq!(active_plan.items[1].meal_type, "lunch");

    // 3. Create second meal plan as active (overrides plan1)
    let plan_req2 = CreateMealPlanRequest {
        name: Some("Bulk Week 2".to_string()),
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 17).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 23).unwrap(),
        is_active: true,
        items: vec![MealPlanItemInput {
            planned_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 17).unwrap(),
            meal_type: "dinner".to_string(),
            recipe_id: Some(recipe.recipe.id),
            ingredient_id: None,
            custom_food_name: None,
            servings: 2.0,
        }],
    };

    let plan2 = plan_service.create_meal_plan(user_id, plan_req2).await.unwrap();
    assert!(plan2.plan.is_active);

    // Verify active plan is now plan2
    let active_plan_new = plan_service.get_active_meal_plan(user_id).await.unwrap();
    assert_eq!(active_plan_new.plan.id, plan2.plan.id);

    // Verify first plan is now deactivated
    let (all_plans, _) = plan_service.list_meal_plans(user_id, 1, 10).await.unwrap();
    let old_plan = all_plans.iter().find(|p| p.id == plan1.plan.id).unwrap();
    assert!(!old_plan.is_active);
}
