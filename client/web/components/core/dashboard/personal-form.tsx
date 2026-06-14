"use client";

import { useEffect, useState } from "react";
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
import { Loader2 } from "lucide-react";
import { toast } from "sonner";

export function PersonalForm() {
  const { profile, updateProfile, biometricLogs, logBiometric } = useProfileStore();

  const [fullName, setFullName] = useState("");
  const [gender, setGender] = useState("");
  const [dob, setDob] = useState("");
  const [timezone, setTimezone] = useState("UTC");
  const [height, setHeight] = useState("");
  const [initialHeight, setInitialHeight] = useState("");
  const [weight, setWeight] = useState("");
  const [initialWeight, setInitialWeight] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (profile) {
      setFullName(profile.full_name || "");
      setGender(profile.gender || "unspecified");
      setDob(profile.dob || "");
      setTimezone(profile.timezone || "UTC");
    }

    const latestHeightLog = biometricLogs.find(
      (log) => log.metric_type === "height_cm" || log.metric_type === "height"
    );
    if (latestHeightLog) {
      const hVal = latestHeightLog.value.toString();
      setHeight(hVal);
      setInitialHeight(hVal);
    }

    const latestWeightLog = biometricLogs.find(
      (log) => log.metric_type === "weight_kg" || log.metric_type === "weight"
    );
    if (latestWeightLog) {
      const wVal = latestWeightLog.value.toString();
      setWeight(wVal);
      setInitialWeight(wVal);
    }
  }, [profile, biometricLogs]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    try {
      await updateProfile({
        full_name: fullName || undefined,
        gender: gender !== "unspecified" ? gender : undefined,
        dob: dob || undefined,
        timezone,
      });

      const hNum = parseFloat(height);
      if (height && !isNaN(hNum)) {
        if (height !== initialHeight) {
          await logBiometric({
            metric_type: "height_cm",
            value: hNum,
            logged_at: new Date().toISOString(),
          });
          setInitialHeight(height);
        }
      }

      const wNum = parseFloat(weight);
      if (weight && !isNaN(wNum)) {
        if (weight !== initialWeight) {
          await logBiometric({
            metric_type: "weight_kg",
            value: wNum,
            logged_at: new Date().toISOString(),
          });
          setInitialWeight(weight);
        }
      }

      toast.success("Profile details updated successfully");
    } catch (err: any) {
      toast.error(err.message || "Failed to update profile");
    } finally {
      setIsSaving(false);
    }
  };

  const getGenderLabel = (g: string) => {
    switch (g) {
      case "unspecified": return "Unspecified";
      case "male": return "Male";
      case "female": return "Female";
      case "other": return "Other";
      default: return g;
    }
  };

  return (
    <div className="border border-border/80 rounded-xl bg-card p-6 flex flex-col gap-6">
      <div className="flex flex-col gap-1 border-b border-border/40 pb-4">
        <h3 className="text-base font-semibold">Personal Details</h3>
        <p className="text-xs text-muted-foreground/85">
          Update your profile name, biological gender, and birth date to personalize planner suggestions.
        </p>
      </div>
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="fullName" className="text-xs font-bold text-muted-foreground">Full Name</Label>
            <Input
              id="fullName"
              placeholder="e.g. Jane Doe"
              value={fullName}
              onChange={(e) => setFullName(e.target.value)}
              className="h-9"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="dob" className="text-xs font-bold text-muted-foreground">Date of Birth</Label>
            <Input
              id="dob"
              type="date"
              value={dob}
              onChange={(e) => setDob(e.target.value)}
              className="h-9"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="gender" className="text-xs font-bold text-muted-foreground">Gender</Label>
            <Select value={gender} onValueChange={(val) => val && setGender(val)}>
              <SelectTrigger className="h-9 w-full rounded-2xl">
                <SelectValue placeholder="Select gender">{getGenderLabel(gender)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="unspecified">Unspecified</SelectItem>
                <SelectItem value="male">Male</SelectItem>
                <SelectItem value="female">Female</SelectItem>
                <SelectItem value="other">Other</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="height" className="text-xs font-bold text-muted-foreground">Height (cm)</Label>
            <Input
              id="height"
              type="number"
              step="any"
              placeholder="e.g. 175"
              value={height}
              onChange={(e) => setHeight(e.target.value)}
              className="h-9"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="weight" className="text-xs font-bold text-muted-foreground">Weight (kg)</Label>
            <Input
              id="weight"
              type="number"
              step="any"
              placeholder="e.g. 70"
              value={weight}
              onChange={(e) => setWeight(e.target.value)}
              className="h-9"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="timezone" className="text-xs font-bold text-muted-foreground">Timezone</Label>
            <Input
              id="timezone"
              placeholder="e.g. Asia/Kolkata"
              value={timezone}
              onChange={(e) => setTimezone(e.target.value)}
              className="h-9"
            />
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
              "Save Profile"
            )}
          </Button>
        </div>
      </form>
    </div>
  );
}
