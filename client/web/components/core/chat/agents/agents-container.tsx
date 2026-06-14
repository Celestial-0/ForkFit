"use client";

import { useChatStore } from "@/store/chat-store";
import { useEffect, useState, useMemo, useCallback } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Calendar as UiCalendar } from "@/components/ui/calendar";
import { 
  Brain, 
  Trash2, 
  ArrowLeft, 
  Calendar, 
  CheckSquare, 
  Square,
  Clock,
  Database,
  History,
  Activity,
  AlertCircle,
  ChefHat,
  BookOpen
} from "lucide-react";
import { cn } from "@/lib/utils";
import { RecipeExpandableCard } from "@/components/core/chat/chatarea/recipe-expandable-card";

const agentStages = [
  { id: "planner", label: "Planner Stage", agents: ["Planner"], isParallel: false },
  { id: "specialists", label: "Specialist Analysis", agents: ["Safety", "Nutrition", "Budget", "Culture"], isParallel: true },
  { id: "recipe", label: "Recipe Formulation", agents: ["Recipe"], isParallel: false },
  { id: "calendar", label: "Calendar Scheduling", agents: ["Calendar"], isParallel: false },
  { id: "shopping", label: "Shopping List Generation", agents: ["Shopping"], isParallel: false },
  { id: "consensus", label: "Consensus Agreement", agents: ["Consensus"], isParallel: false },
  { id: "judge", label: "QA Quality Judge", agents: ["Judge"], isParallel: false },
  { id: "visualization", label: "Visualization Output", agents: ["Visualization"], isParallel: false },
];

const formatDateToYYYYMMDD = (date: Date) => {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

const parseLocalDate = (dateStr: string) => {
  const parts = dateStr.split("-");
  if (parts.length === 3) {
    const y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10) - 1;
    const d = parseInt(parts[2], 10);
    return new Date(y, m, d);
  }
  return new Date(dateStr);
};

interface UiElement {
  type: string;
  title: string;
  config: unknown;
  data: unknown;
}

interface PlanItem {
  planned_date?: string;
  meal_type?: string;
  custom_food_name?: string;
  servings?: number;
  custom_item_name?: string;
  quantity?: number;
  unit?: string;
  category?: string;
  recipe_id?: string;
}

interface PlanData {
  start_date?: string;
  end_date?: string;
  is_active?: boolean;
  items?: PlanItem[];
}

interface RecipeIngredient {
  name: string;
  notes?: string;
  quantity: string;
  unit: string;
  grams_equivalent: number;
}

interface RecipeDetail {
  recipe?: {
    description?: string;
    cuisine?: string;
    course?: string;
    dietary_tags?: string[];
    prep_time_minutes?: number;
    cook_time_minutes?: number;
    servings?: number;
    instructions?: string[];
  };
  detected_allergens?: string[];
  serving_nutrition?: {
    calories: number;
    protein?: number;
    fat?: number;
    carbs?: number;
    fiber?: number;
  };
  serving_estimated_cost?: number;
  ingredients?: RecipeIngredient[];
}

export function AgentsContainer() {
  const {
    steps,
    memories,
    activeTraceId,
    isStreaming,
    fetchActiveMemories,
    deactivateMemory,
    selectedUiElement,
    setSelectedUiElement,
  } = useChatStore();

  const getStageStatus = useCallback((stageAgents: string[]) => {
    const states = stageAgents.map((a) => {
      const step = steps.find((s) => s.agent.toLowerCase() === a.toLowerCase());
      return step ? step.status : "pending";
    });
    if (states.some((s) => s === "running")) return "running";
    if (states.some((s) => s === "failed")) return "failed";
    if (states.every((s) => s === "completed")) return "completed";
    if (states.some((s) => s === "completed")) return "running";
    return "pending";
  }, [steps]);

  // Local state to keep track of checked shopping list items
  const [checkedItems, setCheckedItems] = useState<Record<string, boolean>>({});
  
  // Track the previous UI element to detect changes during render
  const [prevUiElement, setPrevUiElement] = useState<UiElement | null>(null);

  // Interactive calendar selection state
  const [selectedDate, setSelectedDate] = useState<Date | undefined>(undefined);

  // Adjust state when selectedUiElement changes, directly during render
  if (selectedUiElement !== prevUiElement) {
    setPrevUiElement(selectedUiElement);
    
    let initialDate: Date | undefined = undefined;
    if (selectedUiElement && selectedUiElement.type === "meal_plan") {
      const planData = (selectedUiElement.data as PlanData) || {};
      const items = planData.items || [];
      if (items.length > 0) {
        const sortedItems = [...items].sort((a: PlanItem, b: PlanItem) => {
          if (!a.planned_date) return 1;
          if (!b.planned_date) return -1;
          return a.planned_date.localeCompare(b.planned_date);
        });
        const firstDateStr = sortedItems[0]?.planned_date;
        if (firstDateStr) {
          const parts = firstDateStr.split("-");
          if (parts.length === 3) {
            const y = parseInt(parts[0], 10);
            const m = parseInt(parts[1], 10) - 1;
            const d = parseInt(parts[2], 10);
            initialDate = new Date(y, m, d);
          } else {
            initialDate = new Date(firstDateStr);
          }
        }
      }
    }
    setSelectedDate(initialDate);
  }

  useEffect(() => {
    fetchActiveMemories();
  }, [fetchActiveMemories]);

  const toggleCheck = (itemName: string) => {
    setCheckedItems((prev) => ({
      ...prev,
      [itemName]: !prev[itemName],
    }));
  };

  const handleBack = () => {
    setSelectedUiElement(null);
  };

  const planData = useMemo(() => {
    return (selectedUiElement?.data as PlanData) || {};
  }, [selectedUiElement]);

  const planItems = useMemo(() => {
    return planData.items || [];
  }, [planData]);

  const allGrouped = useMemo(() => {
    return planItems.reduce((acc: Record<string, PlanItem[]>, item: PlanItem) => {
      const dateStr = item.planned_date;
      if (dateStr) {
        if (!acc[dateStr]) acc[dateStr] = [];
        acc[dateStr].push(item);
      }
      return acc;
    }, {});
  }, [planItems]);

  const entriesToRender = useMemo(() => {
    const selectedDateStr = selectedDate ? formatDateToYYYYMMDD(selectedDate) : "";
    return selectedDate
      ? Object.entries(allGrouped).filter(([dateStr]) => dateStr === selectedDateStr)
      : Object.entries(allGrouped);
  }, [allGrouped, selectedDate]);

  const recipeDetail = useMemo<RecipeDetail | null>(() => {
    if (selectedUiElement?.type === "recipe") {
      return selectedUiElement.data as RecipeDetail;
    }
    return null;
  }, [selectedUiElement]);

  // 1. RENDER INTERACTIVE UI DETAIL
  if (selectedUiElement) {
    const isMealPlan = selectedUiElement.type === "meal_plan";
    const isRecipe = selectedUiElement.type === "recipe";

    return (
      <div className="flex flex-col h-full bg-card/10 text-foreground overflow-hidden">
        {/* Detail Header */}
        <div className="h-14 border-b flex items-center gap-3 px-4 shrink-0 bg-background/50">
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={handleBack}
            className="text-muted-foreground hover:text-foreground cursor-pointer"
          >
            <ArrowLeft className="size-4" />
          </Button>
          <div className="min-w-0">
            <span className="text-[10px] font-bold text-primary uppercase tracking-wider block">
              Detail Panel View
            </span>
            <h2 className="text-xs font-semibold truncate text-foreground/90 mt-0.5">
              {selectedUiElement.title}
            </h2>
          </div>
        </div>

        {/* Scrollable details */}
        <ScrollArea className="flex-1 min-h-0">
          <div className="p-4 space-y-6">
            {isMealPlan ? (
              // Meal Plan Calendar View
              <div className="space-y-5">
                <div className="bg-muted/10 p-3 rounded-xl border space-y-2">
                  <div className="flex justify-between items-center text-xs">
                    <span className="text-muted-foreground">Schedule Period</span>
                    <span className="font-semibold">
                      {planData.start_date} to {planData.end_date}
                    </span>
                  </div>
                  <div className="flex justify-between items-center text-xs border-t pt-2 mt-2">
                    <span className="text-muted-foreground">Status</span>
                    <Badge variant={planData.is_active ? "default" : "secondary"} className="text-[9px] px-2 py-0.5 font-bold">
                      {planData.is_active ? "Active Diet Program" : "Inactive"}
                    </Badge>
                  </div>
                </div>

                {/* Interactive Calendar Component */}
                <div className="flex flex-col items-center gap-3">
                  <UiCalendar
                    mode="single"
                    selected={selectedDate}
                    onSelect={setSelectedDate}
                    modifiers={{
                      hasMeal: (date: Date) => {
                        const dateStr = formatDateToYYYYMMDD(date);
                        return !!(planData.items || []).some(
                          (item: PlanItem) => item.planned_date === dateStr
                        );
                      }
                    }}
                    modifiersClassNames={{
                      hasMeal: "font-semibold text-primary border border-primary/20 bg-primary/5 hover:bg-primary/15"
                    }}
                    className="rounded-xl border bg-card/45 backdrop-blur-xs p-3.5 shadow-2xs w-full max-w-70"
                  />
                  <div className="flex items-center gap-4 text-[10px] text-muted-foreground select-none">
                    <div className="flex items-center gap-1.5">
                      <div className="size-2 rounded-full bg-primary/10 border border-primary/30" />
                      <span>Scheduled Days</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <div className="size-2 rounded-full bg-primary" />
                      <span>Selected Day</span>
                    </div>
                  </div>
                </div>

                <div className="space-y-3 pt-2">
                  <div className="flex justify-between items-center px-1">
                    <h3 className="text-xs font-bold text-muted-foreground tracking-wider uppercase">
                      {selectedDate 
                        ? `Meals for ${selectedDate.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' })}` 
                        : "All Scheduled Meals"}
                    </h3>
                    {selectedDate ? (
                      <Button
                        variant="ghost"
                        size="xs"
                        onClick={() => setSelectedDate(undefined)}
                        className="text-[10px] h-6 px-2 text-primary hover:text-primary/85 hover:bg-primary/5 cursor-pointer font-semibold"
                      >
                        Show All Days
                      </Button>
                    ) : null}
                  </div>
                  
                  {entriesToRender.length > 0 ? (
                    entriesToRender.map(([dateStr, items]: [string, PlanItem[]]) => (
                      <Card key={dateStr} className="border shadow-2xs overflow-hidden">
                        <CardHeader className="p-3 bg-muted/20 border-b">
                          <CardTitle className="text-xs font-semibold flex items-center gap-1.5">
                            <Calendar className="size-3.5 text-primary" />
                            <span>{parseLocalDate(dateStr).toLocaleDateString(undefined, { weekday: 'long', month: 'short', day: 'numeric' })}</span>
                          </CardTitle>
                        </CardHeader>
                        <CardContent className="p-3 divide-y divide-border/50">
                          {items.map((item: PlanItem, idx: number) => (
                            <div key={idx} className="py-2.5 first:pt-0 last:pb-0 flex justify-between items-start gap-4 text-xs">
                              <div className="space-y-1 min-w-0 flex-1">
                                <span className="text-[10px] uppercase font-bold text-muted-foreground tracking-wider block">
                                  {item.meal_type}
                                </span>
                                {item.recipe_id ? (
                                  <RecipeExpandableCard
                                    recipeId={item.recipe_id}
                                    initialTitle={item.custom_food_name || "Recipe"}
                                    initialDescription="Detailed Recipe Guide"
                                    customTrigger={
                                      <span className="font-semibold text-primary hover:underline flex items-center gap-1.5 text-left cursor-pointer">
                                        <BookOpen className="size-3 text-primary/70 shrink-0" />
                                        <span className="truncate">{item.custom_food_name || "View Recipe"}</span>
                                      </span>
                                    }
                                  />
                                ) : (
                                  <span className="font-semibold text-foreground block truncate">
                                    {item.custom_food_name || "Custom Recipe"}
                                  </span>
                                )}
                              </div>
                              <div className="shrink-0 text-right">
                                <span className="text-muted-foreground">Servings: </span>
                                <span className="font-bold text-primary">{item.servings}</span>
                              </div>
                            </div>
                          ))}
                        </CardContent>
                      </Card>
                    ))
                  ) : (
                    <div className="text-center py-8 px-4 bg-muted/5 border border-dashed rounded-xl space-y-2.5">
                      <p className="text-xs text-muted-foreground font-medium">No meals scheduled for this day.</p>
                      <Button
                        variant="outline"
                        size="xs"
                        onClick={() => {
                          // Reset selection to show all days
                          setSelectedDate(undefined);
                        }}
                        className="text-[10px] h-7 px-3 font-semibold cursor-pointer"
                      >
                        Show All Meals
                      </Button>
                    </div>
                  )}
                </div>
              </div>
            ) : isRecipe && recipeDetail ? (
              // Recipe Detail View
              <div className="space-y-6 animate-in fade-in duration-300">
                {/* Recipe Description & Info */}
                <div className="space-y-3">
                  {recipeDetail.recipe?.description ? (
                    <p className="text-xs text-muted-foreground leading-relaxed italic border-l-2 pl-3">
                      {recipeDetail.recipe.description}
                    </p>
                  ) : null}

                  {/* Dietary Badges */}
                  <div className="flex flex-wrap gap-1.5 pt-1">
                    {recipeDetail.recipe?.cuisine ? (
                      <Badge variant="outline" className="text-[9px] font-bold uppercase tracking-wider border-primary/20 bg-primary/5 text-primary rounded-xs">
                        {recipeDetail.recipe.cuisine}
                      </Badge>
                    ) : null}
                    {recipeDetail.recipe?.course ? (
                      <Badge variant="outline" className="text-[9px] font-bold uppercase tracking-wider border-muted-foreground/20 bg-muted/10 text-muted-foreground rounded-xs">
                        {recipeDetail.recipe.course}
                      </Badge>
                    ) : null}
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
                            isVeg && "bg-[#EDF3EC] text-[#346538] border-[#EDF3EC]/50",
                            isHighProtein && "bg-[#E1F3FE] text-[#1F6C9F] border-[#E1F3FE]/50",
                            isKeto && "bg-[#FBF3DB] text-[#956400] border-[#FBF3DB]/50"
                          )}
                        >
                          {tag}
                        </Badge>
                      );
                    })}
                  </div>

                  {/* Prep/Cook/Servings/Cost Bento Grid */}
                  <div className="grid grid-cols-2 gap-2 pt-2">
                    <Card className="border shadow-3xs p-3 bg-muted/5 flex items-center gap-2.5 rounded-md">
                      <Clock className="size-4 text-primary shrink-0" />
                      <div className="min-w-0">
                        <span className="text-[9px] font-bold text-muted-foreground uppercase tracking-wider block">Time</span>
                        <span className="text-xs font-semibold text-foreground truncate block">
                          {recipeDetail.recipe?.prep_time_minutes || 0}m prep | {recipeDetail.recipe?.cook_time_minutes || 0}m cook
                        </span>
                      </div>
                    </Card>
                    <Card className="border shadow-3xs p-3 bg-muted/5 flex items-center gap-2.5 rounded-md">
                      <ChefHat className="size-4 text-primary shrink-0" />
                      <div className="min-w-0">
                        <span className="text-[9px] font-bold text-muted-foreground uppercase tracking-wider block">Servings & Cost</span>
                        <span className="text-xs font-semibold text-foreground truncate block">
                          {recipeDetail.recipe?.servings || 0} serv | ₹{recipeDetail.serving_estimated_cost?.toFixed(2) || "0.00"}
                        </span>
                      </div>
                    </Card>
                  </div>
                </div>

                {/* Allergen Alert */}
                {recipeDetail.detected_allergens && recipeDetail.detected_allergens.length > 0 ? (
                  <div className="bg-[#FDEBEC] border border-[#9F2F2D]/20 p-3 rounded-lg flex gap-2.5 items-start">
                    <AlertCircle className="size-4 text-[#9F2F2D] shrink-0 mt-0.5" />
                    <div>
                      <span className="text-[10px] font-bold text-[#9F2F2D] uppercase tracking-wider block">Allergens Detected</span>
                      <p className="text-xs text-[#9F2F2D] mt-0.5 font-semibold">
                        This recipe contains: {recipeDetail.detected_allergens.join(", ")}
                      </p>
                    </div>
                  </div>
                ) : null}

                {/* Nutrition Summary */}
                {recipeDetail.serving_nutrition ? (
                  <div className="space-y-2">
                    <h4 className="text-[10px] font-bold text-muted-foreground tracking-wider uppercase px-1">
                      Nutrition Facts (Per Serving)
                    </h4>
                    <Card className="border shadow-2xs overflow-hidden rounded-md">
                      <div className="p-3 bg-muted/10 border-b flex justify-between items-baseline">
                        <span className="text-xs font-semibold text-muted-foreground">Calories</span>
                        <span className="text-base font-extrabold text-primary">{Math.round(recipeDetail.serving_nutrition.calories)} kcal</span>
                      </div>
                      <CardContent className="p-3 grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
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
                ) : null}

                {/* Ingredients */}
                {recipeDetail.ingredients && recipeDetail.ingredients.length > 0 ? (
                  <div className="space-y-2">
                    <h4 className="text-[10px] font-bold text-muted-foreground tracking-wider uppercase px-1">
                      Ingredients
                    </h4>
                    <Card className="border shadow-2xs rounded-md">
                      <CardContent className="p-3 divide-y divide-border/50">
                        {recipeDetail.ingredients.map((ing: RecipeIngredient, idx: number) => (
                          <div key={idx} className="py-2 flex justify-between items-start text-xs first:pt-0 last:pb-0">
                            <div className="min-w-0 pr-2">
                              <span className="font-semibold text-foreground block">{ing.name}</span>
                              {ing.notes ? (
                                <span className="text-[10px] text-muted-foreground italic leading-normal block mt-0.5">
                                  {ing.notes}
                                </span>
                              ) : null}
                            </div>
                            <div className="text-right shrink-0">
                              <span className="font-bold text-primary">{ing.quantity} {ing.unit}</span>
                              {ing.grams_equivalent > 0 && ing.unit !== "grams" && ing.unit !== "g" ? (
                                <span className="text-[10px] text-muted-foreground block mt-0.5">
                                  (~{Math.round(ing.grams_equivalent)}g)
                                </span>
                              ) : null}
                            </div>
                          </div>
                        ))}
                      </CardContent>
                    </Card>
                  </div>
                ) : null}

                {/* Instructions */}
                {recipeDetail.recipe?.instructions && recipeDetail.recipe.instructions.length > 0 ? (
                  <div className="space-y-2">
                    <h4 className="text-[10px] font-bold text-muted-foreground tracking-wider uppercase px-1">
                      Preparation Steps
                    </h4>
                    <div className="space-y-2.5">
                      {recipeDetail.recipe.instructions.map((step: string, idx: number) => (
                        <Card key={idx} className="border shadow-3xs hover:border-primary/15 transition-colors rounded-md">
                          <CardContent className="p-3 flex gap-3 text-xs items-start">
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
                ) : null}
              </div>
            ) : (
              // Shopping List Checked List View
              <div className="space-y-4">
                <div className="bg-muted/10 p-3 rounded-xl border flex justify-between items-center text-xs">
                  <span className="text-muted-foreground">Total Groceries</span>
                  <span className="font-bold">{planData.items?.length || 0} items needed</span>
                </div>

                <div className="space-y-4">
                  {planData.items && planData.items.length > 0 ? (
                    // Group items by category
                    Object.entries(
                      (planData.items || []).reduce((acc: Record<string, PlanItem[]>, item: PlanItem) => {
                        const cat = item.category || "Other";
                        if (!acc[cat]) acc[cat] = [];
                        acc[cat].push(item);
                        return acc;
                      }, {})
                    ).map(([category, items]: [string, PlanItem[]]) => (
                      <div key={category} className="space-y-2">
                        <h4 className="text-xs font-bold text-muted-foreground tracking-wider uppercase px-1">
                          {category}
                        </h4>
                        <Card className="border shadow-2xs">
                          <CardContent className="p-2 divide-y">
                            {items.map((item: PlanItem, idx: number) => {
                              const itemName = item.custom_item_name || "";
                              const isChecked = !!checkedItems[itemName];
                              return (
                                <div
                                  key={idx}
                                  onClick={() => toggleCheck(itemName)}
                                  className="flex items-center justify-between gap-3 p-2 hover:bg-muted/20 rounded cursor-pointer select-none transition-colors first:pt-1 last:pb-1"
                                >
                                  <div className="flex items-center gap-2.5 min-w-0">
                                    <span className="text-muted-foreground shrink-0">
                                      {isChecked ? (
                                        <CheckSquare className="size-4 text-emerald-500 fill-emerald-55" />
                                      ) : (
                                        <Square className="size-4" />
                                      )}
                                    </span>
                                    <span
                                      className={cn(
                                        "text-xs truncate text-foreground",
                                        isChecked && "line-through text-muted-foreground"
                                      )}
                                    >
                                      {item.custom_item_name}
                                    </span>
                                  </div>
                                  <div className="shrink-0 text-xs font-bold text-primary">
                                    {item.quantity} {item.unit}
                                  </div>
                                </div>
                              );
                            })}
                          </CardContent>
                        </Card>
                      </div>
                    ))
                  ) : (
                    <p className="text-xs text-muted-foreground text-center py-4">No shopping items listed.</p>
                  )}
                </div>
              </div>
            )}
          </div>
        </ScrollArea>
      </div>
    );
  }

  // 2. RENDER ACTIVE RUNTIME AND MEMORIES
  return (
    <div className="flex flex-col h-full bg-card/10 text-foreground overflow-hidden">
      {/* Sidebar Header */}
      <div className="h-14 border-b flex items-center px-4 shrink-0 bg-background/50">
        <Activity className="size-4.5 text-primary shrink-0 mr-2" />
        <h2 className="text-sm font-semibold text-foreground">
          Agent Runtime Context
        </h2>
      </div>

      <ScrollArea className="flex-1 min-h-0">
        <div className="p-4 space-y-6">
          {/* Active execution timeline */}
          <div className="space-y-3">
            <h3 className="text-xs font-bold text-muted-foreground tracking-wider uppercase flex items-center gap-1.5">
              <History className="size-3.5" />
              <span>Orchestrator Execution</span>
            </h3>

            {steps.length > 0 ? (
              <Card className="border shadow-2xs">
                <CardContent className="p-3 space-y-3.5">
                  <div className="flex justify-between items-center text-xs pb-2 border-b">
                    <span className="text-muted-foreground">Trace ID</span>
                    <span className="font-mono text-[10px] text-foreground bg-muted px-1.5 py-0.5 rounded truncate max-w-44">
                      {activeTraceId}
                    </span>
                  </div>

                  <div className="relative pl-5 border-l space-y-6 ml-2 pt-1 pb-1">
                    {agentStages.map((stage) => {
                      const status = getStageStatus(stage.agents);
                      const isCompleted = status === "completed";
                      const isRunning = status === "running";
                      const isFailed = status === "failed";
                      const isPending = status === "pending";

                      return (
                        <div key={stage.id} className="relative text-xs">
                          {/* Timeline dot */}
                          <div
                            className={cn(
                              "absolute left-[-26.5px] top-1 size-3 rounded-full border bg-background flex items-center justify-center transition-all duration-300",
                              isCompleted && "bg-emerald-500 border-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.3)]",
                              isRunning && "bg-blue-500 border-blue-500 animate-pulse shadow-[0_0_8px_rgba(59,130,246,0.5)]",
                              isFailed && "bg-destructive border-destructive",
                              isPending && "border-border bg-muted/65"
                            )}
                          >
                            {isCompleted ? <span className="size-1 bg-white rounded-full" /> : null}
                            {isRunning ? <span className="size-1 bg-white rounded-full" /> : null}
                          </div>

                          <div className="space-y-1.5">
                            <div className="flex justify-between items-center">
                              <span className={cn(
                                "font-bold tracking-wider uppercase text-[9.5px]",
                                isPending ? "text-muted-foreground/50" : "text-foreground/90"
                              )}>
                                {stage.label}
                              </span>
                              {!isPending ? (
                                <Badge
                                  variant="secondary"
                                  className={cn(
                                    "text-[8px] uppercase font-bold px-1.5 py-0 h-4 shrink-0 rounded-xs select-none",
                                    isCompleted && "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/10",
                                    isRunning && "bg-blue-500/10 text-blue-600 dark:text-blue-400 border border-blue-500/10 animate-pulse",
                                    isFailed && "bg-destructive/10 text-destructive border border-destructive/10"
                                  )}
                                >
                                  {status}
                                </Badge>
                              ) : null}
                            </div>

                            {stage.isParallel ? (
                              <div className="grid grid-cols-2 gap-1.5 mt-1 w-full">
                                {stage.agents.map((agentName) => {
                                  const agentState = steps.find((s) => s.agent.toLowerCase() === agentName.toLowerCase()) || { status: "pending", latency_ms: undefined };
                                  const isAgentCompleted = agentState.status === "completed";
                                  const isAgentRunning = agentState.status === "running";
                                  const isAgentFailed = agentState.status === "failed";
                                  const isAgentPending = agentState.status === "pending";

                                  return (
                                    <div
                                      key={agentName}
                                      className={cn(
                                        "p-2 rounded border text-[10px] transition-all duration-300 bg-card/25 select-none",
                                        isAgentCompleted && "border-emerald-500/20 bg-emerald-500/5",
                                        isAgentRunning && "border-blue-500/30 bg-blue-500/5 animate-pulse",
                                        isAgentFailed && "border-destructive/20 bg-destructive/5",
                                        isAgentPending && "border-border/40 opacity-40"
                                      )}
                                    >
                                      <div className="flex justify-between items-center gap-1.5">
                                        <span className="font-semibold text-foreground/95 capitalize truncate">
                                          {agentName.replace(/_/g, " ")}
                                        </span>
                                        <div className="flex items-center gap-1 shrink-0">
                                          {isAgentCompleted ? <span className="size-1 rounded-full bg-emerald-500" /> : null}
                                          {isAgentRunning ? <span className="size-1 rounded-full bg-blue-500 animate-ping" /> : null}
                                          {isAgentFailed ? <span className="size-1 rounded-full bg-destructive" /> : null}
                                          {agentState.latency_ms ? (
                                            <span className="text-[8px] text-muted-foreground">
                                              {agentState.latency_ms}ms
                                            </span>
                                          ) : null}
                                        </div>
                                      </div>
                                    </div>
                                  );
                                })}
                              </div>
                            ) : (
                              (() => {
                                const agentName = stage.agents[0];
                                const agentState = steps.find((s) => s.agent.toLowerCase() === agentName.toLowerCase());
                                if (!agentState) return null;

                                return (
                                  <div className="flex justify-between items-center text-[10px] text-muted-foreground bg-muted/5 p-1.5 rounded border border-border/40">
                                    <span className="capitalize">{agentName.replace(/_/g, " ")} execution</span>
                                    {agentState.latency_ms ? <span>{agentState.latency_ms}ms</span> : null}
                                  </div>
                                );
                              })()
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </CardContent>
              </Card>
            ) : (
              <div className="text-center py-6 px-4 bg-muted/10 rounded-xl border border-dashed flex flex-col items-center">
                <Brain className={cn("size-6.5 text-muted-foreground/60 mb-2", isStreaming && "animate-pulse")} />
                <p className="text-xs text-muted-foreground font-medium">
                  {isStreaming ? "Connecting to agent..." : "No active orchestrations"}
                </p>
                <p className="text-[10px] text-muted-foreground/75 mt-0.5">
                  Send a prompt to see LangGraph step executions.
                </p>
              </div>
            )}
          </div>

          {/* Active Memories section */}
          <div className="space-y-3">
            <h3 className="text-xs font-bold text-muted-foreground tracking-wider uppercase flex items-center gap-1.5">
              <Database className="size-3.5" />
              <span>User Constraints Memory</span>
            </h3>

            {memories.length > 0 ? (
              <div className="space-y-2">
                <p className="text-[10px] text-muted-foreground/80 px-1 leading-normal">
                  ForkFit stores preferences extracted from conversations to guide future advice.
                </p>
                {memories.map((memory) => (
                  <Card key={memory.id} className="border shadow-2xs group hover:bg-muted/10 transition-colors">
                    <CardContent className="p-3 flex justify-between items-start gap-4">
                      <div className="space-y-1.5 flex-1 min-w-0">
                        <div className="flex items-center gap-1.5 flex-wrap">
                          <Badge variant="outline" className="text-[9px] uppercase px-1 h-4 font-bold border-primary/20 bg-primary/5 text-primary">
                            {memory.memory_type}
                          </Badge>
                          <span className="text-[9px] text-muted-foreground">
                            Importance: {memory.importance}/5
                          </span>
                        </div>
                        <p className="text-xs font-semibold leading-relaxed text-foreground/90 wrap-break-word pr-2">
                          {memory.content}
                        </p>
                      </div>
                      <button
                        onClick={() => deactivateMemory(memory.id)}
                        className="opacity-0 group-hover:opacity-100 p-1 hover:bg-destructive/10 rounded text-muted-foreground hover:text-destructive shrink-0 transition-opacity cursor-pointer mt-0.5"
                        title="Forget constraint"
                      >
                        <Trash2 className="size-3.5" />
                      </button>
                    </CardContent>
                  </Card>
                ))}
              </div>
            ) : (
              <div className="text-center py-6 px-4 bg-muted/10 rounded-xl border border-dashed">
                <p className="text-xs text-muted-foreground font-medium">No saved constraints</p>
                <p className="text-[10px] text-muted-foreground/75 mt-0.5">
                  AI will build constraints as you chat about dietary targets.
                </p>
              </div>
            )}
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
