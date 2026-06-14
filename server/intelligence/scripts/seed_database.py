import asyncio
import csv
import json
import os
import uuid
import asyncpg
import httpx
import re

def load_dotenv(dot_env_path=".env"):
    if os.path.exists(dot_env_path):
        with open(dot_env_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if "=" in line:
                    key, val = line.split("=", 1)
                    val = val.strip().strip("'\"")
                    if key not in os.environ:
                        os.environ[key] = val

# Try loading from possible .env paths
env_paths = [
    ".env",
    "server/intelligence/.env",
    "server/.env",
    "../.env",
    "../../.env"
]
for path in env_paths:
    if os.path.exists(path):
        load_dotenv(path)

DATABASE_URL = os.environ.get("DATABASE_URL")
CSV_PATH = os.environ.get("CSV_PATH")
OLLAMA_HOST = os.environ.get("OLLAMA_HOST") or os.environ.get("OLLAMA_BASE_URL")
EMBEDDING_MODEL = os.environ.get("EMBEDDING_MODEL")

if not DATABASE_URL:
    raise ValueError("DATABASE_URL is not set in environment or .env file.")
if not CSV_PATH:
    raise ValueError("CSV_PATH is not set in environment or .env file.")
if not OLLAMA_HOST:
    raise ValueError("OLLAMA_HOST or OLLAMA_BASE_URL is not set in environment or .env file.")
if not EMBEDDING_MODEL:
    raise ValueError("EMBEDDING_MODEL is not set in environment or .env file.")

# Load initial ingredient costs from JSON
json_cost_paths = [
    os.path.join(os.path.dirname(__file__), "../../db/seeds/initial_food_costs.json"),
    "db/seeds/initial_food_costs.json",
    "server/db/seeds/initial_food_costs.json",
    "../db/seeds/initial_food_costs.json",
    "../../db/seeds/initial_food_costs.json"
]
costs_file = None
for p in json_cost_paths:
    if os.path.exists(p):
        costs_file = p
        break

INGREDIENT_COSTS = {}
if costs_file:
    print(f"Loading initial ingredient costs from: {costs_file}")
    with open(costs_file, "r", encoding="utf-8") as f:
        INGREDIENT_COSTS = json.load(f)
else:
    print("WARNING: initial_ingredient_costs.json not found. Falling back to default baseline.")

def estimate_cost(food_name: str) -> float:
    if food_name in INGREDIENT_COSTS:
        return INGREDIENT_COSTS[food_name]
    
    # Fallback to case-insensitive or substring matching if not exact match
    name_lower = food_name.lower()
    for key, cost in INGREDIENT_COSTS.items():
        key_lower = key.lower()
        if key_lower == name_lower or key_lower in name_lower or name_lower in key_lower:
            return cost
            
    return 12.50  # Default cost per 100g

async def get_embedding(text: str) -> list[float]:
    url = f"{OLLAMA_HOST}/api/embed"
    try:
        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.post(
                url,
                json={"model": EMBEDDING_MODEL, "input": text}
            )
            if resp.status_code == 200:
                vector = resp.json()["embeddings"][0]
                if len(vector) > 1536:
                    vector = vector[:1536]
                return vector
    except Exception as e:
        print(f"Failed to generate embedding for: {text[:30]}, Error: {e}")
    return [0.0] * 1536

async def seed_raw_food_costs(conn: asyncpg.Connection) -> dict[str, str]:
    print("Seeding raw food costs from JSON...")
    pattern_to_uuid = {}
    for pattern, cost in INGREDIENT_COSTS.items():
        cost_uuid = await conn.fetchval(
            """
            INSERT INTO raw_food_costs (food_pattern, cost_per_100g, price_currency)
            VALUES ($1, $2, 'INR')
            ON CONFLICT (food_pattern) DO UPDATE SET
                cost_per_100g = EXCLUDED.cost_per_100g
            RETURNING id
            """,
            pattern,
            float(cost)
        )
        pattern_to_uuid[pattern.lower().strip()] = str(cost_uuid)
    print(f"Successfully seeded {len(pattern_to_uuid)} raw food costs.")
    return pattern_to_uuid

def find_matched_cost_id(food_name: str, pattern_to_uuid: dict[str, str]) -> uuid.UUID | None:
    food_name_lower = food_name.lower().strip()
    if food_name_lower in pattern_to_uuid:
        return uuid.UUID(pattern_to_uuid[food_name_lower])
    
    # Check substring matches
    for pattern, uuid_str in pattern_to_uuid.items():
        if pattern in food_name_lower or food_name_lower in pattern:
            return uuid.UUID(uuid_str)
    return None

# Global pattern to UUID dictionary populated during main()
PATTERN_TO_UUID = {}

async def seed_food_items(conn: asyncpg.Connection, pattern_to_uuid: dict[str, str]):
    print("Ingesting food items from CSV...")
    if not os.path.exists(CSV_PATH):
        # Try alternative relative paths
        alt_paths = [
            "../db/seeds/Anuvaad_INDB_2024.11.csv",
            "db/seeds/Anuvaad_INDB_2024.11.csv",
            "server/db/seeds/Anuvaad_INDB_2024.11.csv",
            "../../db/seeds/Anuvaad_INDB_2024.11.csv"
        ]
        csv_file = None
        for p in alt_paths:
            if os.path.exists(p):
                csv_file = p
                break
        if not csv_file:
            raise FileNotFoundError(f"CSV file not found at {CSV_PATH}")
    else:
        csv_file = CSV_PATH

    print(f"Reading CSV from: {csv_file}")
    with open(csv_file, mode="r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        count = 0
        portion_count = 0
        
        # Build bulk data list
        for row in reader:
            food_code = row["food_code"]
            food_name = row["food_name"]
            primary_source = row["primarysource"]
            
            # Numeric fields
            energy_kcal = float(row["energy_kcal"]) if row.get("energy_kcal") else 0.0
            protein_g = float(row["protein_g"]) if row.get("protein_g") else 0.0
            carb_g = float(row["carb_g"]) if row.get("carb_g") else 0.0
            fat_g = float(row["fat_g"]) if row.get("fat_g") else 0.0
            fibre_g = float(row["fibre_g"]) if row.get("fibre_g") else 0.0
            sodium_mg = float(row["sodium_mg"]) if row.get("sodium_mg") else 0.0
            
            cost_id = find_matched_cost_id(food_name, pattern_to_uuid)
            
            # Build micronutrients jsonb
            micro = {}
            for col in reader.fieldnames:
                if col not in ["food_code", "food_name", "primarysource", "servings_unit"] and not col.startswith("unit_serving"):
                    val_str = row.get(col)
                    if val_str:
                        try:
                            val = float(val_str)
                            if val > 0:
                                micro[col] = val
                        except ValueError:
                            pass
            
            # Insert or update food item
            food_id = await conn.fetchval(
                """
                INSERT INTO food_items (
                    name, calories_per_100g, protein_per_100g, carbs_per_100g, 
                    fat_per_100g, fiber_per_100g, sodium_mg_per_100g, micronutrients, 
                    food_code, primary_source, raw_food_cost_id, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
                ON CONFLICT (food_code) DO UPDATE SET
                    name = EXCLUDED.name,
                    calories_per_100g = EXCLUDED.calories_per_100g,
                    protein_per_100g = EXCLUDED.protein_per_100g,
                    carbs_per_100g = EXCLUDED.carbs_per_100g,
                    fat_per_100g = EXCLUDED.fat_per_100g,
                    fiber_per_100g = EXCLUDED.fiber_per_100g,
                    sodium_mg_per_100g = EXCLUDED.sodium_mg_per_100g,
                    micronutrients = EXCLUDED.micronutrients,
                    primary_source = EXCLUDED.primary_source,
                    raw_food_cost_id = EXCLUDED.raw_food_cost_id,
                    updated_at = NOW()
                RETURNING id
                """,
                food_name, energy_kcal, protein_g, carb_g, fat_g, fibre_g, sodium_mg, json.dumps(micro),
                food_code, primary_source, cost_id
            )
            count += 1
            
            # Portion configurations
            serv_unit = row.get("servings_unit")
            if serv_unit and serv_unit.strip():
                serv_unit = serv_unit.strip()
                unit_kcal = float(row["unit_serving_energy_kcal"]) if row.get("unit_serving_energy_kcal") else 0.0
                unit_carb = float(row["unit_serving_carb_g"]) if row.get("unit_serving_carb_g") else 0.0
                unit_protein = float(row["unit_serving_protein_g"]) if row.get("unit_serving_protein_g") else 0.0
                unit_fat = float(row["unit_serving_fat_g"]) if row.get("unit_serving_fat_g") else 0.0
                
                grams_equiv = 100.0
                if energy_kcal > 0 and unit_kcal > 0:
                    grams_equiv = round((unit_kcal / energy_kcal) * 100, 2)
                elif carb_g > 0 and unit_carb > 0:
                    grams_equiv = round((unit_carb / carb_g) * 100, 2)
                elif protein_g > 0 and unit_protein > 0:
                    grams_equiv = round((unit_protein / protein_g) * 100, 2)
                elif fat_g > 0 and unit_fat > 0:
                    grams_equiv = round((unit_fat / fat_g) * 100, 2)
                
                await conn.execute(
                    """
                    INSERT INTO food_item_portions (food_item_id, serving_unit, grams_equivalent, created_at, updated_at)
                    VALUES ($1, $2, $3, NOW(), NOW())
                    ON CONFLICT (food_item_id, serving_unit) DO UPDATE SET
                        grams_equivalent = EXCLUDED.grams_equivalent,
                        updated_at = NOW()
                    """,
                    food_id, serv_unit, grams_equiv
                )
                portion_count += 1
                
        print(f"Successfully seeded {count} food items and {portion_count} portion configurations.")

def parse_ingredient_string(ing_str: str):
    ing_str = ing_str.strip()
    if not ing_str:
        return None
        
    # Match leading quantity (number/fraction/decimals)
    match = re.match(r'^([\d\s./½⅓¼¾]+)?\s*(.*)$', ing_str)
    qty_str = match.group(1) if match else None
    rest = match.group(2) if match else ing_str
    
    qty = 100.0  # default quantity in grams if not parsed
    if qty_str:
        qty_str = qty_str.strip()
        try:
            if '/' in qty_str:
                parts = qty_str.split()
                if len(parts) == 2:
                    whole = float(parts[0])
                    frac = parts[1].split('/')
                    qty = whole + float(frac[0]) / float(frac[1])
                else:
                    frac = qty_str.split('/')
                    qty = float(frac[0]) / float(frac[1])
            elif any(c in qty_str for c in ['½', '⅓', '¼', '¾']):
                frac_map = {'½': 0.5, '⅓': 0.33, '¼': 0.25, '¾': 0.75}
                qty = 0.0
                for char in qty_str:
                    if char.isdigit():
                        qty += float(char)
                    elif char in frac_map:
                        qty += frac_map[char]
            else:
                qty = float(qty_str)
        except ValueError:
            qty = 1.0  # fallback
            
    if qty <= 0:
        qty = 1.0
    elif qty > 9999.99:
        qty = 9999.99

    units = [
        'tablespoon', 'tablespoons', 'tbsp', 'teaspoon', 'teaspoons', 'tsp',
        'cup', 'cups', 'g', 'gram', 'grams', 'ml', 'piece', 'pieces', 'pinch', 'pinches',
        'clove', 'cloves', 'slice', 'slices', 'sprig', 'sprigs', 'bunch', 'bunches',
        'inch', 'inches', 'can', 'cans', 'packet', 'packets', 'opt', 'optional'
    ]
    
    unit = 'g' # default
    name_query = rest
    
    words = rest.split()
    if words:
        first_word = words[0].lower().strip(',.()')
        if first_word in units:
            unit = first_word
            name_query = ' '.join(words[1:])
            
    name_query = name_query.strip().strip('-–—,./() ')
    if not name_query:
        name_query = ing_str
        
    name_query = name_query[:100]
    
    # Calculate grams_equivalent
    grams_equivalent = qty
    unit_lower = unit.lower()
    if 'tbsp' in unit_lower or 'tablespoon' in unit_lower:
        grams_equivalent = qty * 15.0
        unit = 'tbsp'
    elif 'tsp' in unit_lower or 'teaspoon' in unit_lower:
        grams_equivalent = qty * 5.0
        unit = 'tsp'
    elif 'cup' in unit_lower:
        grams_equivalent = qty * 200.0
        unit = 'cup'
    elif 'clove' in unit_lower:
        grams_equivalent = qty * 5.0
        unit = 'clove'
    elif 'g' == unit_lower or 'gram' in unit_lower:
        grams_equivalent = qty
        unit = 'g'
    elif 'ml' in unit_lower:
        grams_equivalent = qty
        unit = 'ml'
    else:
        grams_equivalent = qty * 50.0  # assume 50g per unit if unknown
        if unit == 'g':
            unit = 'piece'
            
    if grams_equivalent > 9999.99:
        grams_equivalent = 9999.99
        
    return {
        "name_query": name_query,
        "qty": qty,
        "unit": unit,
        "grams_equivalent": grams_equivalent
    }

async def seed_recipes(conn: asyncpg.Connection):
    recipe_csv = os.environ.get("RECIPE_CSV_PATH", "server/db/seeds/IndianFoodDatasetCSV.csv")
    if not os.path.exists(recipe_csv):
        alt_paths = [
            "../db/seeds/IndianFoodDatasetCSV.csv",
            "db/seeds/IndianFoodDatasetCSV.csv",
            "server/db/seeds/IndianFoodDatasetCSV.csv",
            "../../db/seeds/IndianFoodDatasetCSV.csv"
        ]
        for p in alt_paths:
            if os.path.exists(p):
                recipe_csv = p
                break
                
    if not os.path.exists(recipe_csv):
        print(f"Recipe CSV not found at {recipe_csv}. Skipping recipe seeding.")
        return
        
    print(f"Seeding recipes from {recipe_csv}...")
    
    # Cache existing embeddings to preserve them
    print("Caching existing public recipe embeddings to avoid regeneration...")
    existing_embeddings = {}
    try:
        embedding_rows = await conn.fetch(
            """
            SELECT r.title, e.embedding, e.chunk_text 
            FROM recipes r 
            JOIN recipe_embeddings e ON r.id = e.recipe_id 
            WHERE r.owner_id IS NULL
            """
        )
        for row in embedding_rows:
            title = row["title"]
            existing_embeddings[title.lower().strip()] = {
                "embedding": row["embedding"],
                "chunk_text": row["chunk_text"]
            }
        print(f"Cached {len(existing_embeddings)} recipe embeddings.")
    except Exception as e:
        print(f"No existing recipe embeddings found or error reading them: {e}")

    # Clear existing public recipes
    print("Clearing existing system/public recipes...")
    await conn.execute("DELETE FROM recipes WHERE owner_id IS NULL")
    
    # Check if Ollama is running
    ollama_running = False
    try:
        async with httpx.AsyncClient(timeout=3.0) as client:
            resp = await client.get(f"{OLLAMA_HOST}/api/tags")
            if resp.status_code == 200:
                ollama_running = True
                print("Ollama is active. Recipe vector embeddings will be generated.")
    except Exception as e:
        print("Ollama is offline or unreachable. Seeding will skip vector embedding calls (using zero vectors).")

    # Cache existing ingredients
    print("Caching existing food items from database...")
    ing_rows = await conn.fetch("SELECT id, name FROM food_items")
    ingredient_cache = {}
    for r in ing_rows:
        ingredient_cache[r["name"].lower().strip()] = r["id"]
    print(f"Loaded {len(ingredient_cache)} food items into cache.")

    def find_ingredient_id(name_query):
        q = name_query.lower().strip()
        if not q:
            return None
        
        # 1. Exact match
        if q in ingredient_cache:
            return ingredient_cache[q]
        
        # 2. Substring match
        for name, ing_id in ingredient_cache.items():
            if name in q or q in name:
                return ing_id
                
        # 3. Clean and match words
        ignore_words = {"fresh", "organic", "ground", "powder", "chopped", "sliced", "dried", "paste", "grated", "crushed", "finely", "to", "taste", "as", "required", "extracted", "boiled"}
        words = [w for w in re.split(r'[\s/()\-–—]+', q) if w and w not in ignore_words]
        for w in words:
            if len(w) > 2:
                for name, ing_id in ingredient_cache.items():
                    if w in name:
                        return ing_id
        return None

    # Load limit
    try:
        limit = int(os.environ.get("SEED_RECIPE_LIMIT", "200"))
    except ValueError:
        limit = 200
    seed_all = os.environ.get("SEED_ALL_RECIPES", "").lower() == "true"
    
    with open(recipe_csv, mode="r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        count = 0
        
        for row in reader:
            if not seed_all and count >= limit:
                print(f"Reached default seeding limit of {limit} recipes. Set SEED_ALL_RECIPES=true to seed all recipes.")
                break
                
            title = row.get("TranslatedRecipeName", "").strip() or row.get("RecipeName", "").strip()
            if not title:
                continue
                
            # Check if recipe already exists
            exist_id = await conn.fetchval(
                "SELECT id FROM recipes WHERE title = $1",
                title
            )
            if exist_id:
                print(f"Recipe '{title}' already exists. Skipping.")
                continue
                
            cuisine = row.get("Cuisine", "").strip()
            course = row.get("Course", "").strip()
            diet = row.get("Diet", "").strip()
            source_url = row.get("URL", "").strip()
            
            description = f"A delicious {cuisine or 'Universal'} {course or 'dish'} recipe characterized by its {diet or 'unique flavor'} style."
            
            try:
                prep_time = int(row.get("PrepTimeInMins", 0))
            except ValueError:
                prep_time = 0
                
            try:
                cook_time = int(row.get("CookTimeInMins", 0))
            except ValueError:
                cook_time = 0
                
            try:
                servings = float(row.get("Servings", 1.0))
                if servings <= 0:
                    servings = 1.0
                elif servings > 99.99:
                    servings = 99.99
            except ValueError:
                servings = 1.0
                
            dietary_tags = []
            if diet:
                dietary_tags.append(diet)
                
            raw_instructions = row.get("TranslatedInstructions", "").strip() or row.get("Instructions", "").strip()
            instructions = []
            for step in re.split(r'(?<!\w\.\w.)(?<![A-Z][a-z]\.)(?<=\.|\n)\s*', raw_instructions):
                step = step.strip()
                if step and len(step) > 3:
                    instructions.append(step)
            if not instructions:
                instructions = [raw_instructions] if raw_instructions else ["No instructions provided."]

            # Create recipe
            recipe_id = uuid.uuid4()
            await conn.execute(
                """
                INSERT INTO recipes (
                    id, owner_id, parent_recipe_id, title, description, instructions, 
                    prep_time_minutes, cook_time_minutes, servings, cuisine, course, dietary_tags, source_url, is_public, created_at, updated_at
                )
                VALUES ($1, NULL, NULL, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true, NOW(), NOW())
                """,
                recipe_id, title, description, instructions,
                prep_time, cook_time, servings, cuisine, course, dietary_tags, source_url
            )
            
            # Look up and link ingredients
            raw_ingredients = row.get("TranslatedIngredients", "").strip() or row.get("Ingredients", "").strip()
            for ing_str in raw_ingredients.split(','):
                ing = parse_ingredient_string(ing_str)
                if not ing:
                    continue
                    
                ing_id = find_ingredient_id(ing["name_query"])
                if not ing_id:
                    ing_id = uuid.uuid4()
                    food_code = f"DUMMY_{ing['name_query'].replace(' ', '_').upper()[:30]}_{uuid.uuid4().hex[:8]}"
                    cost_id = find_matched_cost_id(ing["name_query"], PATTERN_TO_UUID)
                    ing_id = await conn.fetchval(
                        """
                        INSERT INTO food_items (
                            id, name, calories_per_100g, protein_per_100g, carbs_per_100g, 
                            fat_per_100g, fiber_per_100g, sodium_mg_per_100g, food_code, raw_food_cost_id
                        )
                        VALUES ($1, $2, 120.0, 8.0, 10.0, 5.0, 1.0, 100.0, $3, $4)
                        ON CONFLICT (name) DO UPDATE SET updated_at = EXCLUDED.updated_at
                        RETURNING id
                        """,
                        ing_id, ing["name_query"], food_code, cost_id
                    )
                    ingredient_cache[ing["name_query"].lower().strip()] = ing_id
                
                await conn.execute(
                    """
                    INSERT INTO recipe_food_items (recipe_id, food_item_id, quantity, unit, grams_equivalent)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (recipe_id, food_item_id) DO NOTHING
                    """,
                    recipe_id, ing_id, ing["qty"], ing["unit"], ing["grams_equivalent"]
                )
                
            # Embed
            cached = existing_embeddings.get(title.lower().strip())
            if cached:
                embedding_val = cached["embedding"]
                chunk_text = cached["chunk_text"]
                if isinstance(embedding_val, (list, tuple)):
                    embedding_str = f"[{','.join(map(str, embedding_val))}]"
                elif hasattr(embedding_val, "tolist"): # numpy array
                    embedding_str = f"[{','.join(map(str, embedding_val.tolist()))}]"
                else:
                    embedding_str = str(embedding_val)
                    
                await conn.execute(
                    """
                    INSERT INTO recipe_embeddings (recipe_id, embedding, chunk_text, updated_at)
                    VALUES ($1, $2::vector, $3, NOW())
                    ON CONFLICT (recipe_id) DO UPDATE
                    SET embedding = EXCLUDED.embedding,
                        chunk_text = EXCLUDED.chunk_text,
                        updated_at = NOW()
                    """,
                    recipe_id, embedding_str, chunk_text
                )
            else:
                embedding = [0.0] * 1536
                if ollama_running:
                    chunk_text = f"Recipe: {title}\nDescription: {description}"
                    print(f"Generating embedding for '{title}'...")
                    embedding = await get_embedding(chunk_text)
                    
                await conn.execute(
                    """
                    INSERT INTO recipe_embeddings (recipe_id, embedding, chunk_text, updated_at)
                    VALUES ($1, $2::vector, $3, NOW())
                    ON CONFLICT (recipe_id) DO UPDATE
                    SET embedding = EXCLUDED.embedding,
                        chunk_text = EXCLUDED.chunk_text,
                        updated_at = NOW()
                    """,
                    recipe_id, f"[{','.join(map(str, embedding))}]", f"Recipe: {title}\nDescription: {description}"
                )
            count += 1
            if count % 10 == 0:
                print(f"Seeded {count} recipes...")
                
    print(f"Successfully seeded {count} recipes from CSV.")

async def main():
    conn = await asyncpg.connect(DATABASE_URL)
    try:
        global PATTERN_TO_UUID
        # Run food items renaming migration
        migration_file = "server/db/migrations/00009_rename_ingredients_to_food_items.sql"
        if not os.path.exists(migration_file):
            alt_paths = [
                "../db/migrations/00009_rename_ingredients_to_food_items.sql",
                "db/migrations/00009_rename_ingredients_to_food_items.sql",
                "server/db/migrations/00009_rename_ingredients_to_food_items.sql",
                "../../db/migrations/00009_rename_ingredients_to_food_items.sql"
            ]
            for p in alt_paths:
                if os.path.exists(p):
                    migration_file = p
                    break
        if os.path.exists(migration_file):
            print(f"Applying migration from {migration_file}...")
            with open(migration_file, 'r', encoding='utf-8') as mf:
                sql = mf.read()
                await conn.execute(sql)
            print("Migration applied successfully.")

        PATTERN_TO_UUID = await seed_raw_food_costs(conn)
        await seed_food_items(conn, PATTERN_TO_UUID)
        await seed_recipes(conn)
    finally:
        await conn.close()

if __name__ == "__main__":
    asyncio.run(main())
