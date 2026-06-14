"use client";

import * as React from "react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { cn } from "@/lib/utils";
import { useAuthStore } from "@/store/auth-store";
import { fetchRecipeDetailApi, RecipeDetailResponse } from "@/lib/api/chat";
import { 
  Clock, 
  ChefHat, 
  AlertCircle, 
  BookOpen, 
  Loader2, 
  X, 
  CheckSquare, 
  Square 
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent,} from "@/components/ui/card";
import { toast } from "sonner";

function getRecipeImage(title: string): string {
  const t = title.toLowerCase();
  if (t.includes("chicken") || t.includes("meat") || t.includes("mutton") || t.includes("fish") || t.includes("egg")) {
    return "https://images.unsplash.com/photo-1598515214211-89d3e73ae83b?w=800&auto=format&fit=crop&q=80"; // Chicken/Meat
  }
  if (t.includes("salad") || t.includes("veg") || t.includes("bowl") || t.includes("health")) {
    return "https://images.unsplash.com/photo-1512621776951-a57141f2eefd?w=800&auto=format&fit=crop&q=80"; // Salad/Healthy
  }
  if (t.includes("dessert") || t.includes("sweet") || t.includes("peda") || t.includes("cake") || t.includes("pastry") || t.includes("halwa") || t.includes("pudding") || t.includes("barfi")) {
    return "https://images.unsplash.com/photo-1587314168485-3236d6710814?w=800&auto=format&fit=crop&q=80"; // Dessert
  }
  if (t.includes("rice") || t.includes("biryani") || t.includes("pulao") || t.includes("khichdi")) {
    return "https://images.unsplash.com/photo-1512058564366-18510be2db19?w=800&auto=format&fit=crop&q=80"; // Rice/Pulao
  }
  if (t.includes("bread") || t.includes("naan") || t.includes("roti") || t.includes("paratha") || t.includes("flatbread") || t.includes("toast")) {
    return "https://images.unsplash.com/photo-1509440159596-0249088772ff?w=800&auto=format&fit=crop&q=80"; // Bread/Roti
  }
  if (t.includes("drink") || t.includes("smoothie") || t.includes("juice") || t.includes("tea") || t.includes("coffee") || t.includes("beverage")) {
    return "https://images.unsplash.com/photo-1536935338788-846bb9981813?w=800&auto=format&fit=crop&q=80"; // Drink
  }
  if (t.includes("curry") || t.includes("paneer") || t.includes("masala") || t.includes("dal") || t.includes("gravy") || t.includes("soup") || t.includes("shorba") || t.includes("sambhar")) {
    return "https://images.unsplash.com/photo-1565557623262-b51c2513a641?w=800&auto=format&fit=crop&q=80"; // Curry/Soup
  }
  return "https://images.unsplash.com/photo-1490645935967-10de6ba17061?w=800&auto=format&fit=crop&q=80"; // Default ingredients table
}

interface RecipeExpandableCardProps {
  recipeId: string;
  initialTitle: string;
  initialDescription?: string;
  className?: string;
  customTrigger?: React.ReactNode;
}

export function RecipeExpandableCard({
  recipeId,
  initialTitle,
  initialDescription = "Recipe Guide",
  className,
  customTrigger,
}: RecipeExpandableCardProps) {
  const [active, setActive] = React.useState(false);
  const [recipeDetail, setRecipeDetail] = React.useState<RecipeDetailResponse | null>(null);
  const [loading, setLoading] = React.useState(false);
  const cardRef = React.useRef<HTMLDivElement>(null);
  const id = React.useId();

  // Local checkbox state for ingredients in the expanded popout
  const [checkedIngredients, setCheckedIngredients] = React.useState<Record<string, boolean>>({});

  const token = useAuthStore((state) => state.accessToken);

  // Fetch recipe details on demand when the card is opened
  const loadRecipeDetails = React.useCallback(async () => {
    if (recipeDetail || !token || !recipeId) return;

    setLoading(true);
    try {
      const details = await fetchRecipeDetailApi(token, recipeId);
      setRecipeDetail(details);
    } catch (err: unknown) {
      console.error(err);
      toast.error("Failed to load recipe details. Please try again.");
      setActive(false);
    } finally {
      setLoading(false);
    }
  }, [recipeId, token, recipeDetail]);

  const handleOpen = () => {
    setActive(true);
    loadRecipeDetails();
  };

  const toggleIngredientCheck = (name: string) => {
    setCheckedIngredients((prev) => ({
      ...prev,
      [name]: !prev[name],
    }));
  };

  React.useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setActive(false);
      }
    };

    const handleClickOutside = (event: MouseEvent | TouchEvent) => {
      if (cardRef.current && !cardRef.current.contains(event.target as Node)) {
        setActive(false);
      }
    };

    if (active) {
      window.addEventListener("keydown", onKeyDown);
      document.addEventListener("mousedown", handleClickOutside);
      document.addEventListener("touchstart", handleClickOutside);
    }

    return () => {
      window.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("touchstart", handleClickOutside);
    };
  }, [active]);

  const recipeImage = getRecipeImage(initialTitle);

  return (
    <>
      {/* Backdrop blur overlay */}
      <AnimatePresence>
        {active && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 h-full w-full bg-black/60 backdrop-blur-md"
          />
        )}
      </AnimatePresence>

      {/* Expanded Modal view */}
      <AnimatePresence>
        {active && (
          <div className="fixed inset-0 z-100 grid place-items-center p-4 sm:p-6 md:p-10 overflow-y-auto">
            <motion.div
              layoutId={`card-${recipeId}-${id}`}
              ref={cardRef}
              className="relative flex h-full max-h-[85vh] w-full max-w-180 flex-col overflow-hidden rounded-3xl border border-border bg-card text-card-foreground shadow-2xl"
            >
              {/* Floating Close Button */}
              <button
                onClick={() => setActive(false)}
                className="absolute right-4 top-4 z-50 flex size-9 items-center justify-center rounded-full border border-border/80 bg-background/80 text-muted-foreground backdrop-blur-xs transition-colors hover:bg-muted hover:text-foreground cursor-pointer"
                aria-label="Close details"
              >
                <X className="size-4" />
              </button>

              {/* Cover Image */}
              <motion.div layoutId={`image-${recipeId}-${id}`} className="relative h-64 shrink-0 overflow-hidden">
                <div className="absolute inset-0 bg-linear-to-t from-card via-card/10 to-transparent z-10" />
                <Image
                  src={recipeImage}
                  alt={initialTitle}
                  className="h-full w-full object-cover object-center"
                />
                <div className="absolute bottom-4 left-6 z-20 pr-12">
                  <motion.p
                    layoutId={`desc-${recipeId}-${id}`}
                    className="text-[10px] font-bold text-primary uppercase tracking-wider block"
                  >
                    {initialDescription}
                  </motion.p>
                  <motion.h3
                    layoutId={`title-${recipeId}-${id}`}
                    className="text-2xl font-extrabold text-foreground tracking-tight mt-0.5"
                  >
                    {initialTitle}
                  </motion.h3>
                </div>
              </motion.div>

              {/* Scrollable Content Area */}
              <div className="flex-1 overflow-y-auto p-6 space-y-6">
                {loading ? (
                  <div className="flex flex-col items-center justify-center py-20 gap-3">
                    <Loader2 className="size-8 text-primary animate-spin" />
                    <p className="text-xs text-muted-foreground font-medium">Gathering culinary secrets...</p>
                  </div>
                ) : recipeDetail ? (
                  <div className="space-y-6 animate-in fade-in duration-200">
                    {/* Description */}
                    {recipeDetail.recipe?.description && (
                      <p className="text-xs text-muted-foreground leading-relaxed italic border-l-2 border-primary/40 pl-3">
                        {recipeDetail.recipe.description}
                      </p>
                    )}

                    {/* Badges */}
                    <div className="flex flex-wrap gap-1.5">
                      {recipeDetail.recipe?.cuisine && (
                        <Badge variant="outline" className="text-[9px] font-bold uppercase tracking-wider border-primary/20 bg-primary/5 text-primary rounded-xs">
                          {recipeDetail.recipe.cuisine}
                        </Badge>
                      )}
                      {recipeDetail.recipe?.course && (
                        <Badge variant="outline" className="text-[9px] font-bold uppercase tracking-wider border-muted-foreground/20 bg-muted/10 text-muted-foreground rounded-xs">
                          {recipeDetail.recipe.course}
                        </Badge>
                      )}
                      {recipeDetail.recipe?.dietary_tags?.map((tag: string) => {
                        const isVeg = tag.toLowerCase().includes("veg") || tag.toLowerCase().includes("plant");
                        const isHighProtein = tag.toLowerCase().includes("protein");
                        const isKeto = tag.toLowerCase().includes("keto") || tag.toLowerCase().includes("low carb");
                        
                        return (
                          <Badge 
                            key={tag} 
                            variant="secondary" 
                            className={cn(
                              "text-[9px] font-bold uppercase tracking-wider rounded-xs border",
                              isVeg && "bg-[#EDF3EC] text-[#346538] border-[#EDF3EC]/50 dark:bg-emerald-950/20 dark:text-emerald-400 dark:border-emerald-950/50",
                              isHighProtein && "bg-[#E1F3FE] text-[#1F6C9F] border-[#E1F3FE]/50 dark:bg-blue-950/20 dark:text-blue-400 dark:border-blue-950/50",
                              isKeto && "bg-[#FBF3DB] text-[#956400] border-[#FBF3DB]/50 dark:bg-amber-950/20 dark:text-amber-400 dark:border-amber-950/50"
                            )}
                          >
                            {tag}
                          </Badge>
                        );
                      })}
                    </div>

                    {/* Prep/Cook/Servings Bento Grid */}
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                      <div className="border shadow-3xs p-3 bg-muted/5 flex items-center gap-2.5 rounded-xl">
                        <Clock className="size-4 text-primary shrink-0" />
                        <div className="min-w-0">
                          <span className="text-[9px] font-bold text-muted-foreground uppercase tracking-wider block">Prep Time</span>
                          <span className="text-xs font-semibold text-foreground truncate block">
                            {recipeDetail.recipe?.prep_time_minutes || 0}m
                          </span>
                        </div>
                      </div>
                      <div className="border shadow-3xs p-3 bg-muted/5 flex items-center gap-2.5 rounded-xl">
                        <Clock className="size-4 text-primary shrink-0" />
                        <div className="min-w-0">
                          <span className="text-[9px] font-bold text-muted-foreground uppercase tracking-wider block">Cook Time</span>
                          <span className="text-xs font-semibold text-foreground truncate block">
                            {recipeDetail.recipe?.cook_time_minutes || 0}m
                          </span>
                        </div>
                      </div>
                      <div className="border shadow-3xs p-3 bg-muted/5 flex items-center gap-2.5 rounded-xl">
                        <ChefHat className="size-4 text-primary shrink-0" />
                        <div className="min-w-0">
                          <span className="text-[9px] font-bold text-muted-foreground uppercase tracking-wider block">Servings</span>
                          <span className="text-xs font-semibold text-foreground truncate block">
                            {recipeDetail.recipe?.servings || 1} serv
                          </span>
                        </div>
                      </div>
                      <div className="border shadow-3xs p-3 bg-muted/5 flex items-center gap-2.5 rounded-xl">
                        <ChefHat className="size-4 text-primary shrink-0" />
                        <div className="min-w-0">
                          <span className="text-[9px] font-bold text-muted-foreground uppercase tracking-wider block">Est. Cost</span>
                          <span className="text-xs font-semibold text-foreground truncate block">
                            ₹{recipeDetail.serving_estimated_cost?.toFixed(2) || "0.00"}
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* Allergen Warning Banner */}
                    {recipeDetail.detected_allergens && recipeDetail.detected_allergens.length > 0 && (
                      <div className="bg-destructive/10 border border-destructive/20 p-3.5 rounded-xl flex gap-2.5 items-start">
                        <AlertCircle className="size-4.5 text-destructive shrink-0 mt-0.5" />
                        <div>
                          <span className="text-[10px] font-bold text-destructive uppercase tracking-wider block">Allergen Safety Warning</span>
                          <p className="text-xs text-destructive mt-0.5 font-semibold">
                            Contains allergens: {recipeDetail.detected_allergens.join(", ")}
                          </p>
                        </div>
                      </div>
                    )}

                    {/* Nutrition Summary */}
                    {recipeDetail.serving_nutrition && (
                      <div className="space-y-2.5">
                        <h4 className="text-[10px] font-bold text-muted-foreground tracking-wider uppercase px-1">
                          Nutrition Facts (Per Serving)
                        </h4>
                        <Card className="border shadow-2xs overflow-hidden rounded-xl">
                          <div className="p-3 bg-muted/10 border-b flex justify-between items-baseline">
                            <span className="text-xs font-semibold text-muted-foreground">Calories</span>
                            <span className="text-base font-extrabold text-primary">{Math.round(recipeDetail.serving_nutrition.calories)} kcal</span>
                          </div>
                          <CardContent className="p-3.5 grid grid-cols-2 gap-x-6 gap-y-2.5 text-xs">
                            <div className="flex justify-between items-center border-b pb-1">
                              <span className="text-muted-foreground">Protein</span>
                              <span className="font-bold text-foreground">{recipeDetail.serving_nutrition.protein?.toFixed(1)}g</span>
                            </div>
                            <div className="flex justify-between items-center border-b pb-1">
                              <span className="text-muted-foreground">Fats</span>
                              <span className="font-bold text-foreground">{recipeDetail.serving_nutrition.fat?.toFixed(1)}g</span>
                            </div>
                            <div className="flex justify-between items-center border-b pb-1">
                              <span className="text-muted-foreground">Carbs</span>
                              <span className="font-bold text-foreground">{recipeDetail.serving_nutrition.carbs?.toFixed(1)}g</span>
                            </div>
                            <div className="flex justify-between items-center border-b pb-1">
                              <span className="text-muted-foreground">Fiber</span>
                              <span className="font-bold text-foreground">{recipeDetail.serving_nutrition.fiber?.toFixed(1)}g</span>
                            </div>
                          </CardContent>
                        </Card>
                      </div>
                    )}

                    {/* Ingredients Checklist */}
                    {recipeDetail.ingredients && recipeDetail.ingredients.length > 0 && (
                      <div className="space-y-2.5">
                        <h4 className="text-[10px] font-bold text-muted-foreground tracking-wider uppercase px-1">
                          Ingredients Checklist
                        </h4>
                        <Card className="border shadow-2xs rounded-xl">
                          <CardContent className="p-2.5 divide-y divide-border/50">
                            {recipeDetail.ingredients.map((ing, idx) => {
                              const isChecked = !!checkedIngredients[ing.name];
                              return (
                                <div 
                                  key={idx} 
                                  onClick={() => toggleIngredientCheck(ing.name)}
                                  className="py-2.5 px-2 flex justify-between items-start text-xs hover:bg-muted/10 rounded-lg cursor-pointer transition-colors select-none"
                                >
                                  <div className="flex items-start gap-2.5 min-w-0 pr-2">
                                    <span className="text-muted-foreground shrink-0 mt-0.5">
                                      {isChecked ? (
                                        <CheckSquare className="size-4 text-emerald-500 fill-emerald-500/10" />
                                      ) : (
                                        <Square className="size-4 text-muted-foreground/60" />
                                      )}
                                    </span>
                                    <div className="min-w-0">
                                      <span className={cn("font-semibold text-foreground block", isChecked && "line-through text-muted-foreground")}>
                                        {ing.name}
                                      </span>
                                      {ing.notes && (
                                        <span className="text-[10px] text-muted-foreground italic leading-normal block mt-0.5">
                                          {ing.notes}
                                        </span>
                                      )}
                                    </div>
                                  </div>
                                  <div className="text-right shrink-0">
                                    <span className={cn("font-bold text-primary", isChecked && "text-muted-foreground")}>
                                      {ing.quantity} {ing.unit}
                                    </span>
                                    {ing.grams_equivalent > 0 && ing.unit !== "grams" && ing.unit !== "g" && (
                                      <span className="text-[10px] text-muted-foreground block mt-0.5">
                                        (~{Math.round(ing.grams_equivalent)}g)
                                      </span>
                                    )}
                                  </div>
                                </div>
                              );
                            })}
                          </CardContent>
                        </Card>
                      </div>
                    )}

                    {/* Instructions / Steps */}
                    {recipeDetail.recipe?.instructions && recipeDetail.recipe.instructions.length > 0 && (
                      <div className="space-y-2.5">
                        <h4 className="text-[10px] font-bold text-muted-foreground tracking-wider uppercase px-1">
                          Preparation Steps
                        </h4>
                        <div className="space-y-2.5">
                          {recipeDetail.recipe.instructions.map((step, idx) => (
                            <Card key={idx} className="border shadow-3xs hover:border-primary/25 transition-colors rounded-xl">
                              <CardContent className="p-3.5 flex gap-3 text-xs items-start">
                                <span className="text-sm font-mono font-bold text-primary/40 select-none shrink-0 leading-none mt-0.5">
                                  {(idx + 1).toString().padStart(2, "0")}
                                </span>
                                <p className="leading-relaxed text-foreground/90 font-medium">
                                  {step}
                                </p>
                              </CardContent>
                            </Card>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="text-center py-20 bg-muted/5 border border-dashed rounded-2xl">
                    <p className="text-xs text-muted-foreground font-medium">Recipe data unavailable.</p>
                  </div>
                )}
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      {/* Trigger Card (Collapsed View) */}
      {customTrigger ? (
        <motion.div
          layoutId={`card-${recipeId}-${id}`}
          onClick={handleOpen}
          className="cursor-pointer"
        >
          {customTrigger}
        </motion.div>
      ) : (
        <motion.div
          layoutId={`card-${recipeId}-${id}`}
          onClick={handleOpen}
          className={cn(
            "flex cursor-pointer flex-col w-full overflow-hidden rounded-2xl border border-primary/15 bg-card/65 p-3 shadow-xs hover:border-primary/40 transition-all duration-300",
            className
          )}
        >
          <div className="flex flex-col gap-3">
            {/* Visual Thumb Image */}
            <motion.div layoutId={`image-${recipeId}-${id}`} className="relative h-32 w-full rounded-xl overflow-hidden">
              <Image
                src={recipeImage}
                alt={initialTitle}
                className="h-full w-full object-cover object-center"
              />
            </motion.div>
            
            <div className="flex flex-col min-w-0">
              <motion.p
                layoutId={`desc-${recipeId}-${id}`}
                className="text-[9px] font-bold text-primary uppercase tracking-wider"
              >
                {initialDescription}
              </motion.p>
              <motion.h3
                layoutId={`title-${recipeId}-${id}`}
                className="font-bold text-xs text-foreground truncate mt-0.5"
              >
                {initialTitle}
              </motion.h3>
            </div>

            <div className="flex justify-between items-center border-t pt-2 mt-1">
              <span className="text-[10px] text-muted-foreground font-medium">Click to expand</span>
              <span className="text-[10px] font-semibold text-primary flex items-center gap-0.5">
                <BookOpen className="size-3" />
                <span>Full Recipe</span>
              </span>
            </div>
          </div>
        </motion.div>
      )}
    </>
  );
}
