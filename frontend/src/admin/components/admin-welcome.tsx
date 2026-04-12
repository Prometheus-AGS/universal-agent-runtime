import { type FC, useCallback, useEffect, useState } from "react";
import {
  BookOpen,
  Bot,
  CheckCircle2,
  ChevronRight,
  Server,
  SlidersHorizontal,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { useOnboarding } from "@/hooks/use-onboarding";
import { cn } from "@/lib/utils";
import { useOnboardingWelcomeStore } from "@/stores/onboarding-welcome-store";

interface SetupStep {
  id: string;
  label: string;
  description: string;
  icon: FC<{ size?: number; className?: string }>;
  check: () => Promise<boolean>;
  navigateTo: string;
}

const SETUP_STEPS: SetupStep[] = [
  {
    id: "provider",
    label: "Configure an LLM provider",
    description:
      "Connect at least one AI provider (OpenAI, Anthropic, etc.) so agents can generate responses.",
    icon: Server,
    check: () => useOnboardingWelcomeStore.getState().checkConfiguredProviders(),
    navigateTo: "/admin/providers",
  },
  {
    id: "default-model",
    label: "Set a default model",
    description:
      "Choose a default model for your provider. Chat will not work until a model is selected.",
    icon: Bot,
    check: () => useOnboardingWelcomeStore.getState().checkDefaultModel(),
    navigateTo: "/admin/providers",
  },
  {
    id: "knowledge",
    label: "Create a knowledge base",
    description:
      "Upload documents so agents can search and reference your content when answering questions.",
    icon: BookOpen,
    check: () => useOnboardingWelcomeStore.getState().checkKnowledgeBases(),
    navigateTo: "/admin/knowledge",
  },
  {
    id: "settings",
    label: "Review your settings",
    description:
      "Check defaults for context management, resilience policies, and memory behavior.",
    icon: SlidersHorizontal,
    check: async () => {
      // Settings always exist — mark complete if user has visited
      try {
        return localStorage.getItem("uar-onboarding-settings-visited") === "1";
      } catch {
        return false;
      }
    },
    navigateTo: "/admin/settings",
  },
];

interface AdminWelcomeProps {
  onNavigate: (path: string) => void;
}

export const AdminWelcome: FC<AdminWelcomeProps> = ({ onNavigate }) => {
  const { dismissed, dismiss } = useOnboarding("admin-welcome");
  const [stepStatus, setStepStatus] = useState<Record<string, boolean>>({});
  const [checking, setChecking] = useState(true);

  const checkSteps = useCallback(async () => {
    await Promise.resolve();
    setChecking(true);
    const results: Record<string, boolean> = {};
    for (const step of SETUP_STEPS) {
      results[step.id] = await step.check();
    }
    setStepStatus(results);
    setChecking(false);
  }, []);

  useEffect(() => {
    if (dismissed) return;
    const id = window.setTimeout(() => {
      void checkSteps();
    }, 0);
    return () => window.clearTimeout(id);
  }, [dismissed, checkSteps]);

  if (dismissed) return null;

  const completedCount = Object.values(stepStatus).filter(Boolean).length;
  const allDone = completedCount === SETUP_STEPS.length;

  return (
    <div className="border-b border-border bg-card px-4 py-5 sm:px-6">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <h3 className="font-display text-base font-semibold text-foreground">
            {allDone ? "You're all set!" : "Welcome to Universal Agent Runtime"}
          </h3>
          <p className="mt-1 font-body text-sm text-muted-foreground">
            {allDone
              ? "Your runtime is configured and ready to use. You can dismiss this banner."
              : "Get started by completing these setup steps. You can always come back to this later."}
          </p>

          {/* Progress */}
          {!checking && (
            <div className="mt-3 flex items-center gap-2">
              <div className="h-1.5 flex-1 rounded-full bg-muted">
                <div
                  className="h-full rounded-full bg-primary transition-all duration-500"
                  style={{
                    width: `${(completedCount / SETUP_STEPS.length) * 100}%`,
                  }}
                />
              </div>
              <span className="shrink-0 font-mono text-xs text-muted-foreground">
                {completedCount}/{SETUP_STEPS.length}
              </span>
            </div>
          )}

          {/* Steps */}
          <div className="mt-4 flex flex-col gap-2">
            {SETUP_STEPS.map((step) => {
              const done = stepStatus[step.id] ?? false;
              const StepIcon = step.icon;
              return (
                <button
                  key={step.id}
                  type="button"
                  onClick={() => {
                    if (step.id === "settings") {
                      try {
                        localStorage.setItem(
                          "uar-onboarding-settings-visited",
                          "1",
                        );
                      } catch {
                        /* ignore */
                      }
                    }
                    onNavigate(step.navigateTo);
                  }}
                  className={cn(
                    "flex items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors",
                    done
                      ? "border-border/50 bg-muted/30"
                      : "border-border bg-background hover:border-primary/30 hover:bg-primary/5",
                  )}
                >
                  <div
                    className={cn(
                      "flex size-8 shrink-0 items-center justify-center rounded-md",
                      done ? "bg-emerald-500/15" : "bg-primary/10",
                    )}
                  >
                    {done ? (
                      <CheckCircle2 size={14} className="text-emerald-500" />
                    ) : (
                      <StepIcon size={14} className="text-primary" />
                    )}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p
                      className={cn(
                        "font-display text-sm font-medium",
                        done
                          ? "text-muted-foreground line-through"
                          : "text-foreground",
                      )}
                    >
                      {step.label}
                    </p>
                    <p className="font-body text-xs text-muted-foreground">
                      {step.description}
                    </p>
                  </div>
                  {!done && (
                    <ChevronRight
                      size={14}
                      className="shrink-0 text-muted-foreground"
                    />
                  )}
                </button>
              );
            })}
          </div>
        </div>

        {/* Dismiss */}
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7 shrink-0 text-muted-foreground"
          onClick={dismiss}
          aria-label="Dismiss welcome guide"
        >
          <X size={14} />
        </Button>
      </div>
    </div>
  );
};
