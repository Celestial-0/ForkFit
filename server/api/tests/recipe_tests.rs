mod common;

use api::infra::pg::recipe_repo::PgRecipeRepository;
use api::recipe::repository::RecipeRepository;
use api::recipe::service::RecipeService;
use api::recipe::types::{CreateFoodItemRequest, CreateRecipeRequest, RecipeFoodItemInput};

#[tokio::test]
async fn test_recipe_nutrition_and_cost() {
    let (_state, db, _) = common::setup_test_state(false, false).await;
    let (user_id, _) = common::setup_test_user(&db).await;

    let repo = PgRecipeRepository::new(db.clone());
    let service = RecipeService::new(repo.clone());

    // Create raw food costs first
    let raw_cost1 = repo.create_raw_food_cost("Peanut Butter", 10.0, "INR").await.unwrap();
    let raw_cost2 = repo.create_raw_food_cost("Whole Milk", 5.0, "INR").await.unwrap();

    // 1. Create food items
    let food_req1 = CreateFoodItemRequest {
        name: format!("Peanut Butter {}", uuid::Uuid::new_v4()),
        description: Some("Creamy peanut butter".to_string()),
        calories_per_100g: 588.0,
        protein_per_100g: 25.0,
        carbs_per_100g: 20.0,
        fat_per_100g: 50.0,
        fiber_per_100g: Some(6.0),
        sodium_mg_per_100g: Some(429.0),
        micronutrients: Some(serde_json::json!({})),
        barcode: None,
        raw_food_cost_id: Some(raw_cost1.id),
    };

    let food_req2 = CreateFoodItemRequest {
        name: format!("Whole Milk {}", uuid::Uuid::new_v4()),
        description: Some("Organic whole milk".to_string()),
        calories_per_100g: 61.0,
        protein_per_100g: 3.2,
        carbs_per_100g: 4.8,
        fat_per_100g: 3.25,
        fiber_per_100g: Some(0.0),
        sodium_mg_per_100g: Some(44.0),
        micronutrients: Some(serde_json::json!({})),
        barcode: None,
        raw_food_cost_id: Some(raw_cost2.id),
    };

    let food1 = service.create_food_item(food_req1).await.unwrap();
    let food2 = service.create_food_item(food_req2).await.unwrap();

    // 2. Create recipe
    let recipe_req = CreateRecipeRequest {
        parent_recipe_id: None,
        title: "High Protein Peanut Butter Shake".to_string(),
        description: Some("Quick bulking shake".to_string()),
        instructions: vec!["Add ingredients to blender".to_string(), "Blend until smooth".to_string()],
        prep_time_minutes: Some(5),
        cook_time_minutes: Some(0),
        servings: 2.0,
        cuisine: Some("American".to_string()),
        course: None,
        dietary_tags: vec!["HighProtein".to_string(), "Bulking".to_string()],
        source_url: None,
        is_public: true,
        food_items: vec![
            RecipeFoodItemInput {
                food_item_id: food1.id,
                quantity: 50.0,
                unit: "grams".to_string(),
                grams_equivalent: 50.0,
                notes: None,
            },
            RecipeFoodItemInput {
                food_item_id: food2.id,
                quantity: 200.0,
                unit: "grams".to_string(),
                grams_equivalent: 200.0,
                notes: None,
            },
        ],
    };

    let detail = service.create_recipe(user_id, recipe_req).await.unwrap();

    assert_eq!(detail.recipe.title, "High Protein Peanut Butter Shake");
    assert_eq!(detail.recipe.servings, 2.0);
    assert_eq!(detail.total_nutrition.calories, 416.0);
    assert_eq!(detail.total_nutrition.protein, 18.9);
    assert_eq!(detail.total_estimated_cost, 15.0);

    // serving-level nutrients
    assert_eq!(detail.serving_nutrition.calories, 208.0);
    assert_eq!(detail.serving_nutrition.protein, 9.45);
    assert_eq!(detail.serving_estimated_cost, 7.5);

    // Allergens
    assert!(detail.detected_allergens.contains(&"Peanuts".to_string()));
    assert!(detail.detected_allergens.contains(&"Dairy".to_string()));

    let fetched = service.get_recipe(detail.recipe.id).await.unwrap();
    assert_eq!(fetched.total_nutrition.calories, 416.0);
}
