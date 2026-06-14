"use client";

import { useEffect, useState } from "react";
import { useProfileStore } from "@/store/profile-store";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Loader2, Plus, X } from "lucide-react";
import { toast } from "sonner";

export function SafetyForm() {
  const { medicalSafety, updateMedicalSafety } = useProfileStore();

  const [allergies, setAllergies] = useState<string[]>([]);
  const [conditions, setConditions] = useState<string[]>([]);
  const [isPregnant, setIsPregnant] = useState(false);
  const [isLactating, setIsLactating] = useState(false);

  const [newAllergy, setNewAllergy] = useState("");
  const [newCondition, setNewCondition] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (medicalSafety) {
      setAllergies(medicalSafety.allergies || []);
      setConditions(medicalSafety.medical_conditions || []);
      setIsPregnant(medicalSafety.is_pregnant || false);
      setIsLactating(medicalSafety.is_lactating || false);
    }
  }, [medicalSafety]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    try {
      await updateMedicalSafety({
        allergies,
        medical_conditions: conditions,
        is_pregnant: isPregnant,
        is_lactating: isLactating,
      });
      toast.success("Medical safety details saved");
    } catch (err: any) {
      toast.error(err.message || "Failed to save medical details");
    } finally {
      setIsSaving(false);
    }
  };

  const addAllergy = () => {
    const val = newAllergy.trim();
    if (val && !allergies.includes(val)) {
      setAllergies([...allergies, val]);
      setNewAllergy("");
    }
  };

  const removeAllergy = (a: string) => {
    setAllergies(allergies.filter((x) => x !== a));
  };

  const addCondition = () => {
    const val = newCondition.trim();
    if (val && !conditions.includes(val)) {
      setConditions([...conditions, val]);
      setNewCondition("");
    }
  };

  const removeCondition = (c: string) => {
    setConditions(conditions.filter((x) => x !== c));
  };

  return (
    <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-6">
      <div>
        <h3 className="text-base font-semibold">Health & Medical Safety</h3>
        <p className="text-xs text-muted-foreground mt-0.5">
          Critical allergies and dietary settings to screen out recipes automatically.
        </p>
      </div>
      <form onSubmit={handleSubmit} className="flex flex-col gap-6">
        {/* Allergies tag manager */}
        <div className="flex flex-col gap-3">
          <Label className="text-xs font-bold text-muted-foreground">Allergies</Label>
          <div className="flex flex-wrap gap-1.5 min-h-8 p-2 border border-dashed border-border/60 rounded-xl">
            {allergies.length === 0 ? (
              <span className="text-xs text-muted-foreground/60 p-1">No allergies registered.</span>
            ) : (
              allergies.map((allergy) => (
                <Badge
                  key={allergy}
                  variant="secondary"
                  className="gap-1 bg-[#FDEBEC] text-[#9F2F2D] border-none py-1 px-2.5 rounded-lg text-xs"
                >
                  <span>{allergy}</span>
                  <button
                    type="button"
                    onClick={() => removeAllergy(allergy)}
                    className="text-[#9F2F2D]/60 hover:text-[#9F2F2D] cursor-pointer"
                  >
                    <X className="size-3" />
                  </button>
                </Badge>
              ))
            )}
          </div>
          <div className="flex gap-2">
            <Input
              placeholder="Add allergy (e.g. Peanuts, Gluten)"
              value={newAllergy}
              onChange={(e) => setNewAllergy(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  addAllergy();
                }
              }}
              className="h-9 flex-1"
            />
            <Button type="button" variant="outline" onClick={addAllergy} className="h-9 text-xs font-semibold cursor-pointer">
              <Plus className="size-4 mr-1" />
              <span>Add</span>
            </Button>
          </div>
        </div>

        {/* Medical Conditions tag manager */}
        <div className="flex flex-col gap-3">
          <Label className="text-xs font-bold text-muted-foreground">Medical Conditions</Label>
          <div className="flex flex-wrap gap-1.5 min-h-8 p-2 border border-dashed border-border/60 rounded-xl">
            {conditions.length === 0 ? (
              <span className="text-xs text-muted-foreground/60 p-1">No conditions registered.</span>
            ) : (
              conditions.map((cond) => (
                <Badge
                  key={cond}
                  variant="secondary"
                  className="gap-1 bg-[#E1F3FE] text-[#1F6C9F] border-none py-1 px-2.5 rounded-lg text-xs"
                >
                  <span>{cond}</span>
                  <button
                    type="button"
                    onClick={() => removeCondition(cond)}
                    className="text-[#1F6C9F]/60 hover:text-[#1F6C9F] cursor-pointer"
                  >
                    <X className="size-3" />
                  </button>
                </Badge>
              ))
            )}
          </div>
          <div className="flex gap-2">
            <Input
              placeholder="Add condition (e.g. Diabetes, Gout)"
              value={newCondition}
              onChange={(e) => setNewCondition(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  addCondition();
                }
              }}
              className="h-9 flex-1"
            />
            <Button type="button" variant="outline" onClick={addCondition} className="h-9 text-xs font-semibold cursor-pointer">
              <Plus className="size-4 mr-1" />
              <span>Add</span>
            </Button>
          </div>
        </div>

        {/* Pregnancy & Lactation switches */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 border-t border-border/40 pt-4">
          <div className="flex items-center justify-between border p-3.5 rounded-xl bg-card-surface/40">
            <div className="space-y-0.5">
              <Label htmlFor="pregnant" className="text-sm font-semibold">Currently Pregnant</Label>
              <p className="text-[11px] text-muted-foreground leading-normal max-w-xs">
                Adapts macros and restricts hazardous foods like raw fish or soft cheeses.
              </p>
            </div>
            <Switch id="pregnant" checked={isPregnant} onCheckedChange={setIsPregnant} />
          </div>

          <div className="flex items-center justify-between border p-3.5 rounded-xl bg-card-surface/40">
            <div className="space-y-0.5">
              <Label htmlFor="lactating" className="text-sm font-semibold">Currently Lactating</Label>
              <p className="text-[11px] text-muted-foreground leading-normal max-w-xs">
                Increases recommended calorie intake to maintain energy levels.
              </p>
            </div>
            <Switch id="lactating" checked={isLactating} onCheckedChange={setIsLactating} />
          </div>
        </div>

        <div className="flex justify-end border-t border-border/40 pt-4">
          <Button type="submit" disabled={isSaving} className="h-9 text-xs font-semibold px-4 cursor-pointer">
            {isSaving ? (
              <>
                <Loader2 className="size-3.5 animate-spin mr-1.5" />
                Saving...
              </>
            ) : (
              "Save Health Details"
            )}
          </Button>
        </div>
      </form>
    </div>
  );
}
