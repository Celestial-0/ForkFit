"use client";

import { useEffect, useState } from "react";
import { useProfileStore } from "@/store/profile-store";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { useTheme } from "next-themes";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";

export function PreferencesForm() {
  const { setTheme } = useTheme();
  const { preferences, updatePreferences } = useProfileStore();

  const [prefTheme, setPrefTheme] = useState("system");
  const [prefLang, setPrefLang] = useState("en");
  const [prefUnit, setPrefUnit] = useState("metric");
  const [prefCuisine, setPrefCuisine] = useState("standard");
  const [prefDiet, setPrefDiet] = useState("standard");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (preferences) {
      setPrefTheme(preferences.theme || "system");
      setPrefLang(preferences.language || "en");
      setPrefUnit(preferences.measurement_system || "metric");
      setPrefCuisine(preferences.preferences?.preferred_cuisine || "standard");
      setPrefDiet(preferences.preferences?.diet || "standard");
    }
  }, [preferences]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    try {
      await updatePreferences({
        theme: prefTheme,
        language: prefLang,
        measurement_system: prefUnit,
        preferences: {
          ...(preferences?.preferences || {}),
          preferred_cuisine: prefCuisine,
          diet: prefDiet,
        },
      });
      setTheme(prefTheme);
      toast.success("Preferences updated successfully");
    } catch (err: any) {
      toast.error(err.message || "Failed to update preferences");
    } finally {
      setIsSaving(false);
    }
  };

  const getThemeLabel = (t: string) => {
    switch (t) {
      case "light": return "Light";
      case "dark": return "Dark";
      case "system": return "System";
      default: return t;
    }
  };

  const getUnitLabel = (u: string) => {
    switch (u) {
      case "metric": return "Metric (kg, cm, km)";
      case "imperial": return "Imperial (lb, in, mi)";
      default: return u;
    }
  };

  const getLangLabel = (l: string) => {
    switch (l) {
      case "en": return "English";
      case "es": return "Español";
      case "fr": return "Français";
      default: return l;
    }
  };

  const getCuisineLabel = (c: string) => {
    switch (c) {
      case "standard": return "Standard (All Cuisines)";
      case "indian": return "Indian Cuisine";
      case "italian": return "Italian Cuisine";
      case "mexican": return "Mexican Cuisine";
      case "continental": return "Continental Cuisine";
      case "french": return "French Cuisine";
      default: return c;
    }
  };

  const getDietLabel = (d: string) => {
    switch (d) {
      case "standard": return "Non-Vegetarian (All foods)";
      case "vegetarian": return "Vegetarian";
      case "vegan": return "Vegan";
      case "jain": return "Jain";
      default: return d;
    }
  };

  return (
    <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-6">
      <div className="flex flex-col gap-1 border-b border-border/40 pb-4">
        <h3 className="text-base font-semibold">App Preferences</h3>
        <p className="text-xs text-muted-foreground/85">
          Choose your interface theme, measurement units for weight/distance, and language.
        </p>
      </div>
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="flex flex-col gap-2">
            <Label className="text-xs font-bold text-muted-foreground">Theme</Label>
            <Select value={prefTheme} onValueChange={(val) => val && setPrefTheme(val)}>
              <SelectTrigger className="h-9 w-full rounded-2xl">
                <SelectValue>{getThemeLabel(prefTheme)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="light">Light</SelectItem>
                <SelectItem value="dark">Dark</SelectItem>
                <SelectItem value="system">System</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-2">
            <Label className="text-xs font-bold text-muted-foreground">Measurement System</Label>
            <Select value={prefUnit} onValueChange={(val) => val && setPrefUnit(val)}>
              <SelectTrigger className="h-9 w-full rounded-2xl">
                <SelectValue>{getUnitLabel(prefUnit)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="metric">Metric (kg, cm, km)</SelectItem>
                <SelectItem value="imperial">Imperial (lb, in, mi)</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-2">
            <Label className="text-xs font-bold text-muted-foreground">Language</Label>
            <Select value={prefLang} onValueChange={(val) => val && setPrefLang(val)}>
              <SelectTrigger className="h-9 w-full rounded-2xl">
                <SelectValue>{getLangLabel(prefLang)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
                <SelectItem value="es">Español</SelectItem>
                <SelectItem value="fr">Français</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 border-t border-border/40 pt-4 mt-2">
          <div className="flex flex-col gap-2">
            <Label className="text-xs font-bold text-muted-foreground">Preferred Cuisine</Label>
            <Select value={prefCuisine} onValueChange={(val) => val && setPrefCuisine(val)}>
              <SelectTrigger className="h-9 w-full rounded-2xl">
                <SelectValue>{getCuisineLabel(prefCuisine)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="standard">Standard (All Cuisines)</SelectItem>
                <SelectItem value="indian">Indian</SelectItem>
                <SelectItem value="italian">Italian</SelectItem>
                <SelectItem value="mexican">Mexican</SelectItem>
                <SelectItem value="continental">Continental</SelectItem>
                <SelectItem value="french">French</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-2">
            <Label className="text-xs font-bold text-muted-foreground">Dietary Type</Label>
            <Select value={prefDiet} onValueChange={(val) => val && setPrefDiet(val)}>
              <SelectTrigger className="h-9 w-full rounded-2xl">
                <SelectValue>{getDietLabel(prefDiet)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="standard">Non-Vegetarian (All foods)</SelectItem>
                <SelectItem value="vegetarian">Vegetarian</SelectItem>
                <SelectItem value="vegan">Vegan</SelectItem>
                <SelectItem value="jain">Jain</SelectItem>
              </SelectContent>
            </Select>
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
              "Save Preferences"
            )}
          </Button>
        </div>
      </form>
    </div>
  );
}
