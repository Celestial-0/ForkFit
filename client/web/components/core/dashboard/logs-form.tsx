"use client";

import { useState } from "react";
import { useProfileStore } from "@/store/profile-store";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Loader2, Scale, Dumbbell } from "lucide-react";
import { toast } from "sonner";

export function LogsForm() {
  const {
    biometricLogs,
    workoutLogs,
    logBiometric,
    logWorkout,
  } = useProfileStore();

  // Biometrics log form state
  const [bioType, setBioType] = useState("weight_kg");
  const [bioValue, setBioValue] = useState("");
  const [bioNotes, setBioNotes] = useState("");
  const [isLoggingBio, setIsLoggingBio] = useState(false);

  // Workout log form state
  const [workActivity, setWorkActivity] = useState("");
  const [workDuration, setWorkDuration] = useState("");
  const [workCalories, setWorkCalories] = useState("");
  const [workNotes, setWorkNotes] = useState("");
  const [isLoggingWork, setIsLoggingWork] = useState(false);

  const getBioTypeLabel = (t: string) => {
    switch (t) {
      case "weight":
      case "weight_kg": return "Body Weight";
      case "body_fat_percentage":
      case "body_fat_pct": return "Body Fat %";
      case "height":
      case "height_cm": return "Height";
      default: return t;
    }
  };

  const handleLogBiometric = async (e: React.FormEvent) => {
    e.preventDefault();
    const val = parseFloat(bioValue);
    if (isNaN(val)) {
      toast.error("Please enter a valid numeric value");
      return;
    }
    setIsLoggingBio(true);
    try {
      await logBiometric({
        metric_type: bioType,
        value: val,
        notes: bioNotes || undefined,
        logged_at: new Date().toISOString(),
      });
      setBioValue("");
      setBioNotes("");
      toast.success("Biometric entry logged");
    } catch (err: any) {
      toast.error(err.message || "Failed to log biometric");
    } finally {
      setIsLoggingBio(false);
    }
  };

  const handleLogWorkout = async (e: React.FormEvent) => {
    e.preventDefault();
    const dur = parseInt(workDuration);
    const cal = parseInt(workCalories);
    if (!workActivity.trim()) {
      toast.error("Please enter an activity name");
      return;
    }
    if (isNaN(dur) || dur <= 0) {
      toast.error("Please enter a valid duration");
      return;
    }
    if (isNaN(cal) || cal < 0) {
      toast.error("Please enter valid calories");
      return;
    }
    setIsLoggingWork(true);
    try {
      await logWorkout({
        activity_name: workActivity,
        duration_minutes: dur,
        calories_burned: cal,
        notes: workNotes || undefined,
        logged_at: new Date().toISOString(),
      });
      setWorkActivity("");
      setWorkDuration("");
      setWorkCalories("");
      setWorkNotes("");
      toast.success("Workout logged successfully");
    } catch (err: any) {
      toast.error(err.message || "Failed to log workout");
    } finally {
      setIsLoggingWork(false);
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-start">
      {/* BIOMETRICS LOG COLUMN */}
      <div className="flex flex-col gap-6">
        {/* Log form */}
        <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-6">
          <div className="flex items-center gap-2">
            <Scale className="size-5 text-primary" />
            <h3 className="text-base font-semibold">Log Biometrics</h3>
          </div>
          <form onSubmit={handleLogBiometric} className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Metric Type</Label>
              <Select value={bioType} onValueChange={(val) => val && setBioType(val)}>
                <SelectTrigger className="h-9 w-full rounded-2xl">
                  <SelectValue>{getBioTypeLabel(bioType)}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="weight_kg">Body Weight</SelectItem>
                  <SelectItem value="body_fat_pct">Body Fat %</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Value ({bioType === "weight_kg" ? "kg" : "%"})</Label>
              <Input
                type="number"
                step="any"
                placeholder="e.g. 78.5"
                value={bioValue}
                onChange={(e) => setBioValue(e.target.value)}
                required
                className="h-9"
              />
            </div>

            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Notes (Optional)</Label>
              <Input
                placeholder="Add comments..."
                value={bioNotes}
                onChange={(e) => setBioNotes(e.target.value)}
                className="h-9"
              />
            </div>

            <Button type="submit" disabled={isLoggingBio} className="w-full h-9 text-xs font-semibold cursor-pointer">
              {isLoggingBio ? (
                <>
                  <Loader2 className="size-3.5 animate-spin mr-1.5" />
                  Logging...
                </>
              ) : (
                "Log Metric"
              )}
            </Button>
          </form>
        </div>

        {/* Logs list */}
        <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-4">
          <h4 className="text-xs font-bold uppercase text-muted-foreground/80 tracking-wider">Biometric History</h4>
          <div className="flex flex-col gap-2 max-h-80 overflow-y-auto pr-1">
            {biometricLogs.length === 0 ? (
              <p className="text-xs text-muted-foreground/60 py-2">No biometrics logged yet.</p>
            ) : (
              biometricLogs
                .slice()
                .sort((a, b) => new Date(b.logged_at).getTime() - new Date(a.logged_at).getTime())
                .map((log) => (
                  <div key={log.id} className="flex items-center justify-between border border-border/40 p-2.5 rounded-lg text-xs bg-muted/10">
                    <div>
                      <span className="font-bold capitalize">{getBioTypeLabel(log.metric_type)}: </span>
                      <span>
                        {log.value}{" "}
                        {log.metric_type === "weight_kg" || log.metric_type === "weight"
                          ? "kg"
                          : log.metric_type === "height_cm" || log.metric_type === "height"
                          ? "cm"
                          : "%"}
                      </span>
                      {log.notes && <p className="text-[10px] text-muted-foreground mt-0.5">Note: {log.notes}</p>}
                    </div>
                    <span className="text-[10px] text-muted-foreground font-mono shrink-0">
                      {new Date(log.logged_at).toLocaleDateString()}
                    </span>
                  </div>
                ))
            )}
          </div>
        </div>
      </div>

      {/* WORKOUT LOG COLUMN */}
      <div className="flex flex-col gap-6">
        {/* Log form */}
        <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-6">
          <div className="flex items-center gap-2">
            <Dumbbell className="size-5 text-primary" />
            <h3 className="text-base font-semibold">Log Workout Session</h3>
          </div>
          <form onSubmit={handleLogWorkout} className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Activity Name</Label>
              <Input
                placeholder="e.g. Strength Training, Running"
                value={workActivity}
                onChange={(e) => setWorkActivity(e.target.value)}
                required
                className="h-9"
              />
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="flex flex-col gap-2">
                <Label className="text-xs font-bold text-muted-foreground">Duration (mins)</Label>
                <Input
                  type="number"
                  placeholder="e.g. 45"
                  value={workDuration}
                  onChange={(e) => setWorkDuration(e.target.value)}
                  required
                  className="h-9"
                />
              </div>
              <div className="flex flex-col gap-2">
                <Label className="text-xs font-bold text-muted-foreground">Calories (kcal)</Label>
                <Input
                  type="number"
                  placeholder="e.g. 350"
                  value={workCalories}
                  onChange={(e) => setWorkCalories(e.target.value)}
                  required
                  className="h-9"
                />
              </div>
            </div>

            <div className="flex flex-col gap-2">
              <Label className="text-xs font-bold text-muted-foreground">Notes (Optional)</Label>
              <Input
                placeholder="e.g. Felt strong, high energy"
                value={workNotes}
                onChange={(e) => setWorkNotes(e.target.value)}
                className="h-9"
              />
            </div>

            <Button type="submit" disabled={isLoggingWork} className="w-full h-9 text-xs font-semibold cursor-pointer">
              {isLoggingWork ? (
                <>
                  <Loader2 className="size-3.5 animate-spin mr-1.5" />
                  Logging...
                </>
              ) : (
                "Log Workout"
              )}
            </Button>
          </form>
        </div>

        {/* Logs list */}
        <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-4">
          <h4 className="text-xs font-bold uppercase text-muted-foreground/80 tracking-wider">Workout History</h4>
          <div className="flex flex-col gap-2 max-h-80 overflow-y-auto pr-1">
            {workoutLogs.length === 0 ? (
              <p className="text-xs text-muted-foreground/60 py-2">No workouts logged yet.</p>
            ) : (
              workoutLogs
                .slice()
                .sort((a, b) => new Date(b.logged_at).getTime() - new Date(a.logged_at).getTime())
                .map((log) => (
                  <div key={log.id} className="flex flex-col gap-1 border border-border/40 p-2.5 rounded-lg text-xs bg-muted/10">
                    <div className="flex items-center justify-between">
                      <span className="font-bold">{log.activity_name}</span>
                      <span className="text-[10px] text-muted-foreground font-mono">
                        {new Date(log.logged_at).toLocaleDateString()}
                      </span>
                    </div>
                    <p className="text-muted-foreground text-[10px]">
                      Duration: {log.duration_minutes} mins • Calories: {log.calories_burned} kcal
                    </p>
                    {log.notes && <p className="text-[10px] text-muted-foreground mt-0.5 border-t border-border/20 pt-1">Note: {log.notes}</p>}
                  </div>
                ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
