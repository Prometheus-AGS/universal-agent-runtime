import { useId, useRef, useState, type ChangeEvent } from "react";
import { CheckCircle2, FileUp, Info, Loader2, OctagonX } from "lucide-react";
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
  AlertDialogFooter, AlertDialogHeader, AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button, buttonVariants } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import {
  definitionKey,
  useTeamActions,
  type PackagePreflightResponse,
  type TeamsErrorInfo,
} from "@/platform/entities";
import { hasBlocking } from "../model/binding-diagnostics";
import { errorInfoOf } from "../model/errors";
import {
  analysePackageSelection,
  teamDefinitionKeysOf,
  type FileCheckStatus,
  type FileReport,
  type PackageAnalysis,
} from "../model/package-analysis";
import { cn } from "@/lib/utils";
import { useCommandId } from "../model/use-sheet-helpers";
import { DiagnosticList, RuntimeNotice } from "./runtime-notice";
import { SheetActions } from "./sheet-actions";

const CHECK_TEXT: Record<FileCheckStatus, string> = {
  ok: "Matches the manifest",
  missing: "Missing",
  digest: "Digest differs",
  extra: "Not listed",
};

function CheckIcon({ status }: { status: FileCheckStatus }) {
  if (status === "ok") return <CheckCircle2 aria-hidden="true" className="mt-0.5 size-4 shrink-0 text-success" />;
  if (status === "extra") return <Info aria-hidden="true" className="mt-0.5 size-4 shrink-0 text-muted-foreground" />;
  return <OctagonX aria-hidden="true" className="mt-0.5 size-4 shrink-0 text-destructive" />;
}

function FileChecks({ reports }: { reports: readonly FileReport[] }) {
  return (
    <ul aria-label="Chosen files checked against the manifest" className="space-y-2">
      {reports.map((report) => (
        <li key={`${report.status}:${report.path}`} className="flex gap-2 rounded-lg bg-muted/60 p-3 text-sm">
          <CheckIcon status={report.status} />
          <div className="min-w-0">
            <p className="font-mono text-xs break-all">{report.path}</p>
            <p className="font-medium">{CHECK_TEXT[report.status]}</p>
            {report.status !== "ok" && <p className="break-words text-xs text-muted-foreground">{report.detail}</p>}
          </div>
        </li>
      ))}
    </ul>
  );
}

function Analysis({ analysis }: { analysis: PackageAnalysis }) {
  return (
    <section aria-labelledby="install-analysis-heading" className="space-y-3">
      <h3 id="install-analysis-heading" className="text-sm font-semibold">
        {analysis.manifest ? `Manifest ${analysis.manifest.id}@${analysis.manifest.version}` : "Manifest"}
      </h3>
      {analysis.problems.map((problem) => (
        <p key={problem} role="alert" className="rounded-lg bg-destructive/10 p-3 text-sm break-words text-destructive">{problem}</p>
      ))}
      <FileChecks reports={analysis.reports} />
      {analysis.manifest && !analysis.ready && (
        <p role="status" className="text-sm text-muted-foreground">Fix the files above before the package is checked. Nothing has been sent to the runtime.</p>
      )}
    </section>
  );
}

function PreflightResult({ result }: { result: PackagePreflightResponse }) {
  // The runtime marks every team definition as not executable at package level and installs it anyway;
  // binding preflight is where activation is decided. So this result informs, it does not gate.
  const pending = !result.activationSupported || hasBlocking(result.diagnostics);
  const name = `${result.package.id}@${result.package.version}`;
  const count = `${result.definitions.length} ${result.definitions.length === 1 ? "definition" : "definitions"}`;
  return (
    <section aria-labelledby="install-preflight-heading" className="space-y-3">
      <h3 id="install-preflight-heading" className="text-sm font-semibold">Package check</h3>
      <p role="status" className="text-sm">
        {pending
          ? `The runtime accepted ${name} (${count}) and it can be installed. The items marked "Blocks activation" describe the package before it is bound to models: the binding step decides whether a team can run.`
          : `The runtime accepted ${name} (${count}) and reports nothing that blocks activation.`}
      </p>
      <DiagnosticList label="Package check diagnostics" diagnostics={result.diagnostics} />
    </section>
  );
}

interface InstallFormProps {
  onClose: () => void;
  onInstalled: (definitionKey: string | null) => void;
}

function InstallForm({ onClose, onInstalled }: InstallFormProps) {
  const actions = useTeamActions();
  const commandId = useCommandId();
  const generation = useRef(0);
  const fileInputId = useId();
  const footerReasonId = useId();
  const [chosenCount, setChosenCount] = useState(0);
  const [analysis, setAnalysis] = useState<PackageAnalysis | null>(null);
  const [analysing, setAnalysing] = useState(false);
  const [preflight, setPreflight] = useState<PackagePreflightResponse | null>(null);
  const [busy, setBusy] = useState<"preflight" | "install" | null>(null);
  const [error, setError] = useState<TeamsErrorInfo | null>(null);
  const [confirming, setConfirming] = useState(false);

  const choose = async (event: ChangeEvent<HTMLInputElement>) => {
    const list = Array.from(event.target.files ?? []);
    const mine = ++generation.current;
    setChosenCount(list.length);
    setPreflight(null);
    setError(null);
    commandId.renew();
    setAnalysing(true);
    try {
      const chosen = await Promise.all(list.map(async (file) => ({ name: file.name, bytes: new Uint8Array(await file.arrayBuffer()) })));
      const result = await analysePackageSelection(chosen);
      if (mine === generation.current) setAnalysis(result);
    } catch (failure) {
      if (mine === generation.current) {
        setAnalysis({ manifest: null, manifestText: null, files: {}, reports: [], ready: false, problems: [`The files could not be checked: ${errorInfoOf(failure).message}`] });
      }
    } finally {
      if (mine === generation.current) setAnalysing(false);
    }
  };

  const ready = analysis?.ready === true && analysis.manifestText !== null;
  const check = async () => {
    if (!analysis?.ready || analysis.manifestText === null) return;
    const mine = generation.current;
    setBusy("preflight");
    setError(null);
    try {
      const result = await actions.preflightTeamPackage({ manifest: analysis.manifestText, files: analysis.files });
      if (mine === generation.current) setPreflight(result); // A newer file choice owns the result slot.
    } catch (failure) {
      if (mine === generation.current) {
        setPreflight(null);
        setError(errorInfoOf(failure));
      }
    } finally {
      setBusy(null);
    }
  };

  const install = async () => {
    if (!analysis?.manifest || analysis.manifestText === null || !preflight) return;
    setConfirming(false);
    setBusy("install");
    setError(null);
    try {
      const response = await actions.installTeamPackage({
        manifest: analysis.manifestText,
        files: analysis.files,
        commandId: commandId.get(),
      });
      const keys = teamDefinitionKeysOf(analysis.manifest);
      const fallback = response.preflight.definitions[0];
      onInstalled(keys[0] ?? (fallback ? definitionKey(fallback) : null));
    } catch (failure) {
      setError(errorInfoOf(failure));
    } finally {
      setBusy(null);
    }
  };

  const installable = preflight !== null; // A preflight the runtime answered; see PreflightResult for why activation does not gate.
  // The next step is the primary action: Check package until the runtime has answered, then Install package.
  const checkButton = (
    <Button type="button" className="min-h-11" variant={installable ? "secondary" : "default"} disabled={!ready || busy !== null} onClick={() => void check()}>
      {busy === "preflight" && <Loader2 aria-hidden="true" className="motion-safe:animate-spin" />}
      Check package
    </Button>
  );
  const installButton = (
    <Button type="button" className="min-h-11" variant={installable ? "default" : "secondary"} disabled={!installable || busy !== null} onClick={() => setConfirming(true)}>
      {busy === "install" && <Loader2 aria-hidden="true" className="motion-safe:animate-spin" />}
      Install package
    </Button>
  );
  const footerReason = installable || busy !== null
    ? null
    : analysis === null
      ? "Choose the manifest and every file it lists to continue."
      : !ready
        ? "Fix the files listed above to continue."
        : "Check the package before installing it.";
  return (
    <>
      <div className="min-h-0 flex-1 space-y-5 overflow-y-auto px-4">
        <div className="space-y-2">
          <p className="text-sm font-medium">Package files</p>
          <p id={`${fileInputId}-hint`} className="text-sm text-muted-foreground">
            Choose the manifest (kind PackageManifest) together with every file it lists. Each file's digest is checked here before the runtime is asked anything.
          </p>
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
            {/* The real input is visually hidden but stays focusable; its label is the visible button (peer = the label's sibling). */}
            <input id={fileInputId} type="file" multiple accept=".json,application/json" className="peer sr-only"
              aria-describedby={`${fileInputId}-hint`} disabled={busy !== null} onChange={(event) => void choose(event)} />
            <Label
              htmlFor={fileInputId}
              className={cn(
                buttonVariants({ variant: "secondary" }),
                "min-h-11 cursor-pointer peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-ring peer-disabled:pointer-events-none peer-disabled:opacity-50",
              )}
            >
              <FileUp aria-hidden="true" className="size-4" />
              Choose package files
            </Label>
            <span className="text-sm text-muted-foreground">
              {chosenCount === 0 ? "No files chosen" : `${chosenCount} ${chosenCount === 1 ? "file" : "files"} chosen`}
            </span>
          </div>
          {analysing && <p role="status" className="text-sm text-muted-foreground">Checking files…</p>}
        </div>
        {analysis && <Analysis analysis={analysis} />}
        <RuntimeNotice error={error} />
        {preflight && <PreflightResult result={preflight} />}
      </div>
      <SheetActions
        reason={footerReason}
        reasonId={footerReasonId}
        primary={installable ? installButton : checkButton}
        secondary={installable ? checkButton : installButton}
        cancelDisabled={busy === "install"}
        onCancel={onClose}
      />
      <AlertDialog open={confirming} onOpenChange={setConfirming}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Install this package permanently?</AlertDialogTitle>
            <AlertDialogDescription>
              An installed package cannot be removed or edited from the console or the runtime. Its definitions become available to every workspace of this owner.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel className="min-h-11" variant="secondary">Keep checking</AlertDialogCancel>
            <AlertDialogAction className="min-h-11" onClick={() => void install()}>Install permanently</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

interface InstallPackageSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The team definition to select afterwards, if the manifest names one. */
  onInstalled: (definitionKey: string | null) => void;
}

export function InstallPackageSheet({ open, onOpenChange, onInstalled }: InstallPackageSheetProps) {
  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="flex w-full flex-col sm:max-w-xl">
        <SheetHeader>
          <SheetTitle>Install team package</SheetTitle>
          <SheetDescription>Check a package against its manifest and the runtime, then install it.</SheetDescription>
        </SheetHeader>
        {open && <InstallForm onClose={() => onOpenChange(false)} onInstalled={onInstalled} />}
      </SheetContent>
    </Sheet>
  );
}
