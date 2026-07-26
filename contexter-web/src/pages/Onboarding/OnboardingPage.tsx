import { useCallback } from 'react';
import { RefreshCw, Rocket, CheckCircle2, Circle } from 'lucide-react';
import { useOnboardingStatus, useSubmitOnboarding } from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { Button } from '@/components/ui/Button';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';

export function OnboardingPage() {
  const status = useOnboardingStatus();
  const submit = useSubmitOnboarding();

  const isLoading = status.isLoading;
  const isError = status.isError;
  const data = status.data;

  const handleCompleteStep = useCallback(
    (stepId: string) => {
      submit.mutate(stepId);
    },
    [submit],
  );

  const handleRetry = useCallback(() => {
    status.refetch();
  }, [status]);

  /* ── Loading ────────────────────────────────────────────── */
  if (isLoading) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Onboarding" />
        <div className="flex flex-col gap-4">
          <LoadingSkeleton variant="card" count={4} />
        </div>
      </div>
    );
  }

  /* ── Error ──────────────────────────────────────────────── */
  if (isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Onboarding" />
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <Rocket className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">Failed to load onboarding status</h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching your onboarding progress.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  /* ── Completed ──────────────────────────────────────────── */
  if (data?.completed) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Onboarding" />
        <div className="flex flex-col items-center justify-center gap-6 rounded-lg border border-border bg-surface p-12 text-center">
          <div className="rounded-full bg-success/10 p-4">
            <CheckCircle2 className="h-12 w-12 text-success" aria-hidden="true" />
          </div>
          <div>
            <h2 className="text-2xl font-bold text-text-primary">Onboarding Complete!</h2>
            <p className="mt-2 max-w-md text-sm text-text-secondary">
              You've completed all onboarding steps. You're ready to explore the full power of Contexter.
            </p>
          </div>
        </div>
      </div>
    );
  }

  /* ── In Progress ────────────────────────────────────────── */
  const steps = data?.steps ?? [];

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Onboarding" />

      {/* Progress bar */}
      <div className="flex items-center gap-3">
        <div className="h-2 flex-1 overflow-hidden rounded-full bg-bg-tertiary">
          <div
            className="h-full rounded-full bg-accent transition-all duration-500"
            style={{
              width: `${data ? ((data.current_step) / data.total_steps) * 100 : 0}%`,
            }}
          />
        </div>
        <span className="text-sm text-text-secondary">
          Step {data?.current_step ?? 0} of {data?.total_steps ?? 0}
        </span>
      </div>

      {/* Steps */}
      <div className="flex flex-col gap-3">
        {steps.map((step, idx) => {
          const isCurrent = idx + 1 === (data?.current_step ?? 1);
          return (
            <div
              key={step.id}
              className={`flex items-center gap-4 rounded-lg border p-4 transition-colors ${
                step.completed
                  ? 'border-success/30 bg-success/5'
                  : isCurrent
                    ? 'border-accent/30 bg-accent/5'
                    : 'border-border bg-surface'
              }`}
            >
              {/* Status icon */}
              {step.completed ? (
                <CheckCircle2 className="h-6 w-6 shrink-0 text-success" aria-hidden="true" />
              ) : isCurrent ? (
                <Circle className="h-6 w-6 shrink-0 text-accent" aria-hidden="true" />
              ) : (
                <Circle className="h-6 w-6 shrink-0 text-text-tertiary" aria-hidden="true" />
              )}

              {/* Step info */}
              <div className="flex-1">
                <span
                  className={`text-sm font-medium ${
                    step.completed
                      ? 'text-success line-through'
                      : isCurrent
                        ? 'text-text-primary'
                        : 'text-text-tertiary'
                  }`}
                >
                  {step.label}
                </span>
              </div>

              {/* Action button for current step */}
              {isCurrent && !step.completed && (
                <Button
                  variant="primary"
                  onClick={() => handleCompleteStep(step.id)}
                  loading={submit.isPending}
                >
                  Complete Step
                </Button>
              )}

              {/* Completed badge */}
              {step.completed && (
                <span className="text-xs font-medium text-success">Done</span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
