"""Pure nutrition math functions — NO database, NO LLM calls.

Implements BMR, TDEE, macro split calculations, recipe nutrition
aggregation, and plan adherence scoring.
"""
from __future__ import annotations


def calculate_bmr(weight_kg: float, height_cm: float, age: int, gender: str) -> float:
    """Compute Basal Metabolic Rate using the Mifflin-St Jeor equation.

    Args:
        weight_kg: Body weight in kilograms.
        height_cm: Height in centimeters.
        age: Age in years.
        gender: "male"/"m" or "female"/"f".

    Returns:
        BMR in kcal/day.
    """
    base = (10 * weight_kg) + (6.25 * height_cm) - (5 * age)
    if gender.lower() in ("male", "m"):
        return base + 5
    return base - 161


def calculate_tdee(bmr: float, activity_level: str) -> float:
    """Compute Total Daily Energy Expenditure from BMR and activity level.

    Args:
        bmr: Basal Metabolic Rate in kcal/day.
        activity_level: One of sedentary, light, moderate, active, very_active.

    Returns:
        TDEE in kcal/day.
    """
    multipliers = {
        "sedentary": 1.2,
        "light": 1.375,
        "moderate": 1.55,
        "active": 1.725,
        "very_active": 1.9,
    }
    return bmr * multipliers.get(activity_level.lower(), 1.55)


def calculate_macro_split(tdee: float, goal: str) -> dict[str, float]:
    """Calculate daily macro targets based on TDEE and fitness goal.

    Applies a caloric adjustment (deficit/surplus) and distributes
    calories across protein, carbs, and fat.

    Args:
        tdee: Total Daily Energy Expenditure in kcal/day.
        goal: Fitness goal string (weight_loss, muscle_gain, maintenance, etc.).

    Returns:
        Dict with keys: calories, protein_g, carbs_g, fat_g.
    """
    adjustments = {
        "weight_loss": -500,
        "fat_loss": -500,
        "muscle_gain": +300,
        "calorie_surplus": +300,
        "maintenance": 0,
        "macro_maintenance": 0,
    }
    adjusted = tdee + adjustments.get(goal.lower(), 0)

    # Macro ratios vary by goal
    if goal.lower() in ("muscle_gain", "calorie_surplus"):
        protein_ratio, carb_ratio, fat_ratio = 0.30, 0.45, 0.25
    elif goal.lower() in ("weight_loss", "fat_loss"):
        protein_ratio, carb_ratio, fat_ratio = 0.35, 0.35, 0.30
    else:  # maintenance
        protein_ratio, carb_ratio, fat_ratio = 0.25, 0.50, 0.25

    return {
        "calories": round(adjusted, 1),
        "protein_g": round((adjusted * protein_ratio) / 4, 1),   # 4 cal/g
        "carbs_g": round((adjusted * carb_ratio) / 4, 1),        # 4 cal/g
        "fat_g": round((adjusted * fat_ratio) / 9, 1),           # 9 cal/g
    }


def calculate_recipe_nutrition(ingredients: list[dict]) -> dict[str, float]:
    """Sum nutrition from a list of ingredient dicts.

    Each ingredient dict must contain per-100g values and a
    grams_equivalent field indicating the actual quantity used.

    Required keys per ingredient:
        calories_per_100g, protein_per_100g, carbs_per_100g,
        fat_per_100g, fiber_per_100g, sodium_mg_per_100g,
        grams_equivalent.

    Returns:
        Dict with keys: calories, protein_g, carbs_g, fat_g, fiber_g, sodium_mg.
    """
    totals: dict[str, float] = {
        "calories": 0.0,
        "protein_g": 0.0,
        "carbs_g": 0.0,
        "fat_g": 0.0,
        "fiber_g": 0.0,
        "sodium_mg": 0.0,
    }
    for ing in ingredients:
        factor = float(ing.get("grams_equivalent", 0)) / 100.0
        totals["calories"] += float(ing.get("calories_per_100g", 0)) * factor
        totals["protein_g"] += float(ing.get("protein_per_100g", 0)) * factor
        totals["carbs_g"] += float(ing.get("carbs_per_100g", 0)) * factor
        totals["fat_g"] += float(ing.get("fat_per_100g", 0)) * factor
        totals["fiber_g"] += float(ing.get("fiber_per_100g", 0)) * factor
        totals["sodium_mg"] += float(ing.get("sodium_mg_per_100g", 0)) * factor

    return {k: round(v, 1) for k, v in totals.items()}


def score_plan_adherence(plan_nutrition: dict, targets: dict) -> float:
    """Score how well a plan's nutrition matches macro targets.

    Computes a 0.0–1.0 score based on deviation of actual values
    from target values for calories, protein, carbs, and fat.

    Args:
        plan_nutrition: Actual nutrition dict (calories, protein_g, etc.).
        targets: Target nutrition dict with the same keys.

    Returns:
        Adherence score between 0.0 (worst) and 1.0 (perfect match).
    """
    scores: list[float] = []
    for key in ("calories", "protein_g", "carbs_g", "fat_g"):
        target = targets.get(key, 0)
        actual = plan_nutrition.get(key, 0)
        if target <= 0:
            continue
        ratio = actual / target
        # Perfect score at ratio=1.0, linear penalty for deviation
        score = max(0.0, 1.0 - abs(1.0 - ratio))
        scores.append(score)
    return round(sum(scores) / len(scores), 3) if scores else 0.0
