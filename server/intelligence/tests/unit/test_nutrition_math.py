from src.services.nutrition_math import (
    calculate_bmr,
    calculate_tdee,
    calculate_macro_split,
    calculate_recipe_nutrition,
    score_plan_adherence,
)

def test_calculate_bmr():
    # Male test case
    bmr_m = calculate_bmr(75.0, 175.0, 30, "male")
    assert abs(bmr_m - 1698.75) < 0.01

    # Female test case
    bmr_f = calculate_bmr(60.0, 160.0, 25, "female")
    assert abs(bmr_f - 1314.0) < 0.01


def test_calculate_tdee():
    bmr = 1500.0
    assert abs(calculate_tdee(bmr, "sedentary") - 1800.0) < 0.01
    assert abs(calculate_tdee(bmr, "light") - 2062.5) < 0.01
    assert abs(calculate_tdee(bmr, "moderate") - 2325.0) < 0.01
    assert abs(calculate_tdee(bmr, "active") - 2587.5) < 0.01
    assert abs(calculate_tdee(bmr, "very_active") - 2850.0) < 0.01


def test_calculate_macro_split():
    tdee = 2000.0
    
    # Weight loss (500 kcal deficit, 35% P, 35% C, 30% F)
    macros_wl = calculate_macro_split(tdee, "weight_loss")
    assert macros_wl["calories"] == 1500.0
    assert macros_wl["protein_g"] == 131.2 # 1500 * 0.35 / 4 = 131.25
    assert macros_wl["carbs_g"] == 131.2
    assert macros_wl["fat_g"] == 50.0 # 1500 * 0.30 / 9 = 50.0
    
    # Muscle gain (300 kcal surplus, 30% P, 45% C, 25% F)
    macros_mg = calculate_macro_split(tdee, "muscle_gain")
    assert macros_mg["calories"] == 2300.0
    assert macros_mg["protein_g"] == 172.5 # 2300 * 0.30 / 4 = 172.5
    assert macros_mg["carbs_g"] == 258.8 # 2300 * 0.45 / 4 = 258.75
    assert macros_mg["fat_g"] == 63.9 # 2300 * 0.25 / 9 = 63.88


def test_calculate_recipe_nutrition():
    ingredients = [
        {
            "grams_equivalent": 100.0,
            "calories_per_100g": 100.0,
            "protein_per_100g": 10.0,
            "carbs_per_100g": 20.0,
            "fat_per_100g": 2.0,
            "fiber_per_100g": 4.0,
            "sodium_mg_per_100g": 100.0,
        },
        {
            "grams_equivalent": 50.0,
            "calories_per_100g": 200.0,
            "protein_per_100g": 20.0,
            "carbs_per_100g": 10.0,
            "fat_per_100g": 8.0,
            "fiber_per_100g": 2.0,
            "sodium_mg_per_100g": 500.0,
        }
    ]
    breakdown = calculate_recipe_nutrition(ingredients)
    assert breakdown["calories"] == 200.0  # 100 + 100
    assert breakdown["protein_g"] == 20.0   # 10 + 10
    assert breakdown["carbs_g"] == 25.0     # 20 + 5
    assert breakdown["fat_g"] == 6.0       # 2 + 4
    assert breakdown["fiber_g"] == 5.0     # 4 + 1
    assert breakdown["sodium_mg"] == 350.0 # 100 + 250


def test_score_plan_adherence():
    targets = {
        "calories": 2000.0,
        "protein_g": 150.0,
        "carbs_g": 200.0,
        "fat_g": 60.0
    }
    
    # Perfect match
    score_perfect = score_plan_adherence(targets, targets)
    assert score_perfect == 1.0
    
    # Moderate deviation
    actual = {
        "calories": 1800.0,  # 90%
        "protein_g": 135.0, # 90%
        "carbs_g": 220.0,   # 110% (deviation 10%, score 90%)
        "fat_g": 54.0       # 90%
    }
    score_dev = score_plan_adherence(actual, targets)
    assert abs(score_dev - 0.90) < 0.01
