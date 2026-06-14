"use client";

import { useEffect, useState } from "react";
import { useProfileStore } from "@/store/profile-store";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Loader2, Target, Trash2 } from "lucide-react";
import { toast } from "sonner";

export function GoalsForm() {
  const { goals, createGoal, deactivateGoal } = useProfileStore();

  const [goalCategory, setGoalCategory] = useState("weight");
  const [goalTargetType, setGoalTargetType] = useState("lose_weight");
  const [goalTargetValue, setGoalTargetValue] = useState("");
  const [goalUnit, setGoalUnit] = useState("kg");
  const [goalStartDate, setGoalStartDate] = useState("");
  const [goalTargetDate, setGoalTargetDate] = useState("");
  const [isCreatingGoal, setIsCreatingGoal] = useState(false);

  const getCategoryLabel = (cat: string) => {
    switch (cat) {
      case "weight": return "Weight Tracker";
      case "activity": return "Activity (Steps/Workouts)";
      case "nutrition": return "Nutrition (Calories/Water)";
      case "sleep": return "Sleep Duration";
      default: return cat;
    }
  };

  const getTargetTypeLabel = (type: string) => {
    switch (type) {
      case "lose_weight": return "Lose Weight";
      case "gain_weight": return "Gain Weight";
      case "maintain_weight": return "Maintain Weight";
      case "daily_steps": return "Daily Steps";
      case "weekly_workouts": return "Weekly Workouts";
      case "daily_calories": return "Daily Calories";
      case "daily_water": return "Daily Water Intake";
      case "sleep_duration": return "Sleep Duration";
      default: return type;
    }
  };

  useEffect(() => {
    setGoalStartDate(new Date().toISOString().split("T")[0]);
  }, []);

  useEffect(() => {
    if (goalCategory === "weight") {
      setGoalTargetType("lose_weight");
      setGoalUnit("kg");
    } else if (goalCategory === "activity") {
      setGoalTargetType("daily_steps");
      setGoalUnit("steps");
    } else if (goalCategory === "nutrition") {
      setGoalTargetType("daily_calories");
      setGoalUnit("kcal");
    } else if (goalCategory === "sleep") {
      setGoalTargetType("sleep_duration");
      setGoalUnit("hours");
    }
  }, [goalCategory]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const val = parseFloat(goalTargetValue);
    if (isNaN(val)) {
      toast.error("Please enter a valid target value");
      return;
    }
    setIsCreatingGoal(true);
    try {
      await createGoal({
        category: goalCategory,
        target_type: goalTargetType,
        target_value: val,
        unit: goalUnit,
        config: {},
        start_date: goalStartDate || undefined,
        target_date: goalTargetDate || undefined,
      });
      setGoalTargetValue("");
      setGoalTargetDate("");
      toast.success("Fitness goal created successfully");
    } catch (err: any) {
      toast.error(err.message || "Failed to create goal");
    } finally {
      setIsCreatingGoal(false);
    }
  };

  const handleDeactivate = async (category: string) => {
    if (confirm(`Are you sure you want to deactivate your active ${category} goal?`)) {
      try {
        await deactivateGoal(category);
        toast.success("Goal deactivated successfully");
      } catch (err: any) {
        toast.error(err.message || "Failed to deactivate goal");
      }
    }
  };

  const activeGoals = goals.filter((g) => g.is_active);

  return (
    <div className="flex flex-col gap-6">
      {/* Active Goals list */}
      <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-6">
        <h3 className="text-base font-semibold">Active Goals</h3>
        <div className="flex flex-col gap-3">
          {activeGoals.length === 0 ? (
            <div className="border border-dashed border-border p-6 rounded-xl flex flex-col items-center justify-center text-center">
              <Target className="size-8 text-muted-foreground/50 mb-2" />
              <p className="text-sm font-semibold text-muted-foreground">No active goals found</p>
              <p className="text-xs text-muted-foreground/70 mt-0.5">
                Set goals below to align the planner with your target body weight and calories.
              </p>
            </div>
          ) : (
            activeGoals.map((goal) => (
              <div key={goal.id} className="flex items-center justify-between border border-border p-4 rounded-xl">
                <div className="flex items-center gap-3">
                  <div className="size-9 rounded-lg bg-primary/5 border border-primary/10 flex items-center justify-center shrink-0">
                    <Target className="size-5 text-primary" />
                  </div>
                  <div>
                    <div className="text-sm font-bold capitalize flex items-center gap-2">
                      <span>{goal.category} Goal</span>
                      <Badge variant="secondary" className="bg-[#EDF3EC] text-[#346538] border-none text-[10px] px-2 py-0.5 rounded-lg uppercase tracking-wider font-semibold">
                        {goal.target_type.replace("_", " ")}
                      </Badge>
                    </div>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      Target: {goal.target_value} {goal.unit}
                      {goal.target_date && ` • Target Date: ${new Date(goal.target_date).toLocaleDateString()}`}
                    </p>
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleDeactivate(goal.category)}
                  className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 cursor-pointer rounded-lg"
                  aria-label="Deactivate goal"
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Create Goal Form */}
      <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-6">
        <h3 className="text-base font-semibold">Define New Goal</h3>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Category</Label>
              <Select value={goalCategory} onValueChange={(val) => val && setGoalCategory(val)}>
                <SelectTrigger className="h-9 w-full rounded-2xl">
                  <SelectValue>{getCategoryLabel(goalCategory)}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="weight">Weight Tracker</SelectItem>
                  <SelectItem value="activity">Activity (Steps/Workouts)</SelectItem>
                  <SelectItem value="nutrition">Nutrition (Calories/Water)</SelectItem>
                  <SelectItem value="sleep">Sleep Duration</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Goal Target Type</Label>
              {goalCategory === "weight" && (
                <Select value={goalTargetType} onValueChange={(val) => val && setGoalTargetType(val)}>
                  <SelectTrigger className="h-9 w-full rounded-2xl">
                    <SelectValue>{getTargetTypeLabel(goalTargetType)}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="lose_weight">Lose Weight</SelectItem>
                    <SelectItem value="gain_weight">Gain Weight</SelectItem>
                    <SelectItem value="maintain_weight">Maintain Weight</SelectItem>
                  </SelectContent>
                </Select>
              )}
              {goalCategory === "activity" && (
                <Select value={goalTargetType} onValueChange={(val) => val && setGoalTargetType(val)}>
                  <SelectTrigger className="h-9 w-full rounded-2xl">
                    <SelectValue>{getTargetTypeLabel(goalTargetType)}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="daily_steps">Daily Steps</SelectItem>
                    <SelectItem value="weekly_workouts">Weekly Workouts</SelectItem>
                  </SelectContent>
                </Select>
              )}
              {goalCategory === "nutrition" && (
                <Select value={goalTargetType} onValueChange={(val) => val && setGoalTargetType(val)}>
                  <SelectTrigger className="h-9 w-full rounded-2xl">
                    <SelectValue>{getTargetTypeLabel(goalTargetType)}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="daily_calories">Daily Calories</SelectItem>
                    <SelectItem value="daily_water">Daily Water Intake</SelectItem>
                  </SelectContent>
                </Select>
              )}
              {goalCategory === "sleep" && (
                <Select value={goalTargetType} onValueChange={(val) => val && setGoalTargetType(val)}>
                  <SelectTrigger className="h-9 w-full rounded-2xl">
                    <SelectValue>{getTargetTypeLabel(goalTargetType)}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="sleep_duration">Sleep Duration</SelectItem>
                  </SelectContent>
                </Select>
              )}
            </div>

            <div className="flex grid grid-cols-2 gap-2">
              <div className="flex flex-col gap-2">
                <Label className="text-xs font-bold text-muted-foreground">Target Value</Label>
                <Input
                  placeholder="Target"
                  type="number"
                  step="any"
                  value={goalTargetValue}
                  onChange={(e) => setGoalTargetValue(e.target.value)}
                  required
                  className="h-9"
                />
              </div>
              <div className="flex flex-col gap-2">
                <Label className="text-xs font-bold text-muted-foreground">Unit</Label>
                <Input
                  value={goalUnit}
                  onChange={(e) => setGoalUnit(e.target.value)}
                  required
                  className="h-9"
                />
              </div>
            </div>

            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Target Date</Label>
              <Input
                type="date"
                value={goalTargetDate}
                onChange={(e) => setGoalTargetDate(e.target.value)}
                className="h-9"
              />
            </div>
          </div>

          <div className="flex justify-end border-t border-border/40 pt-4">
            <Button type="submit" disabled={isCreatingGoal} className="h-9 text-xs font-semibold px-4 cursor-pointer">
              {isCreatingGoal ? (
                <>
                  <Loader2 className="size-3.5 animate-spin mr-1.5" />
                  Creating...
                </>
              ) : (
                "Create Fitness Goal"
              )}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
