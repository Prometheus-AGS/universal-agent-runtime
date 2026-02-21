import {
    type FC,
    type ReactNode,
    useCallback,
    useEffect,
    useState,
} from "react";
import {
    AlertCircle,
    Bot,
    Brain,
    Check,
    ChevronRight,
    Database,
    Eye,
    EyeOff,
    FileText,
    Layers,
    Loader2,
    RefreshCw,
    Save,
    Scissors,
    Server,
    ShieldCheck,
    SlidersHorizontal,
    Zap,
} from "lucide-react";
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
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { useSettings } from "@/hooks/use-settings";
import type { SettingsType } from "@/types";

// =============================================================================
// Namespace definition — the sidebar nav
// =============================================================================

type NavCategory = "AI & LLM" | "File Processing" | "Infrastructure" | "Governance & Agents";

interface NavItem {
    key: string;
    label: string;
    icon: FC<{ size?: number; className?: string }>;
    category: NavCategory;
}

const NAV_ITEMS: NavItem[] = [
    { key: "provider", label: "LLM Providers", icon: Server, category: "AI & LLM" },
    { key: "vision", label: "Vision", icon: Eye, category: "AI & LLM" },
    { key: "context_management", label: "Context Management", icon: Layers, category: "AI & LLM" },
    { key: "rag", label: "RAG & Chunking", icon: Scissors, category: "AI & LLM" },
    { key: "knowledge_bases", label: "Knowledge Bases", icon: Database, category: "AI & LLM" },
    { key: "memory", label: "Memory", icon: Brain, category: "AI & LLM" },
    { key: "file_processing", label: "File Processing", icon: FileText, category: "File Processing" },
    { key: "unstructured", label: "Unstructured API", icon: FileText, category: "File Processing" },
    { key: "mistral_ocr", label: "Mistral OCR", icon: SlidersHorizontal, category: "File Processing" },
    { key: "kreuzberg", label: "Kreuzberg OCR", icon: SlidersHorizontal, category: "File Processing" },
    { key: "resilience", label: "Resilience", icon: Brain, category: "Infrastructure" },
    { key: "intent_classifier", label: "Intent Classifier", icon: Brain, category: "Infrastructure" },
    { key: "governance", label: "Governance", icon: ShieldCheck, category: "Governance & Agents" },
    { key: "agent_config", label: "Agents", icon: Bot, category: "Governance & Agents" },
    { key: "skill_config", label: "Skills", icon: Zap, category: "Governance & Agents" },
];

const CATEGORIES: NavCategory[] = [
    "AI & LLM",
    "File Processing",
    "Infrastructure",
    "Governance & Agents",
];

// =============================================================================
// Shared primitive components
// =============================================================================

const Field: FC<{ label: string; hint?: string; children: ReactNode }> = ({
    label,
    hint,
    children,
}) => (
    <div className="space-y-1.5">
        <Label className="font-mono text-[11px] font-medium text-muted-foreground uppercase tracking-wide">
            {label}
        </Label>
        {children}
        {hint && (
            <p className="font-mono text-[10px] text-muted-foreground/60">{hint}</p>
        )}
    </div>
);

const Toggle: FC<{
    value: boolean;
    onChange: (v: boolean) => void;
    disabled?: boolean;
}> = ({ value, onChange, disabled }) => (
    <Switch
        checked={value}
        onCheckedChange={onChange}
        disabled={disabled}
    />
);

const MaskedInput: FC<{
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
}> = ({ value, onChange, placeholder }) => {
    const [show, setShow] = useState(false);
    return (
        <div className="flex gap-1.5">
            <Input
                type={show ? "text" : "password"}
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder={placeholder ?? "••••••••"}
                className="font-mono text-[12px]"
            />
            <Button
                variant="ghost"
                size="icon"
                className="shrink-0"
                onClick={() => setShow((s) => !s)}
                type="button"
            >
                {show ? <EyeOff size={14} /> : <Eye size={14} />}
            </Button>
        </div>
    );
};

const SettingSelect: FC<{
    value: string;
    options: { value: string; label: string }[];
    onChange: (v: string) => void;
}> = ({ value, options, onChange }) => (
    <Select value={value ?? ""} onValueChange={onChange}>
        <SelectTrigger className="font-mono text-[12px]">
            <SelectValue />
        </SelectTrigger>
        <SelectContent>
            {options.map((o) => (
                <SelectItem key={o.value} value={o.value} className="font-mono text-[12px]">
                    {o.label}
                </SelectItem>
            ))}
        </SelectContent>
    </Select>
);

function PanelHeader({
    title,
    subtitle,
    saving,
    onSave,
    onReload,
    loading,
    saveDisabled = false,
}: {
    title: string;
    subtitle?: string;
    saving: boolean;
    loading: boolean;
    onSave: () => void;
    onReload: () => void;
    saveDisabled?: boolean;
}) {
    return (
        <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
            <div>
                <h2 className="font-display text-lg font-semibold text-foreground">
                    {title}
                </h2>
                {subtitle && (
                    <p className="font-mono text-[11px] text-muted-foreground">{subtitle}</p>
                )}
            </div>
            <div className="flex items-center gap-2">
                <Button
                    variant="outline"
                    size="sm"
                    onClick={onReload}
                    disabled={loading}
                    className="gap-1.5"
                >
                    <RefreshCw size={13} className={cn(loading && "animate-spin")} />
                    Refresh
                </Button>
                <Button
                    size="sm"
                    onClick={onSave}
                    disabled={saving || saveDisabled}
                    className="gap-1.5"
                >
                    {saving ? (
                        <Loader2 size={13} className="animate-spin" />
                    ) : (
                        <Save size={13} />
                    )}
                    Save
                </Button>
            </div>
        </div>
    );
}

function SavedBanner({ show }: { show: boolean }) {
    if (!show) return null;
    return (
        <div className="mx-6 mb-4 flex items-center gap-2 rounded-lg border border-success/40 bg-success/10 px-4 py-2">
            <Check size={13} className="text-success" />
            <span className="font-mono text-[11px] text-success">Settings saved</span>
        </div>
    );
}

function ErrorBanner({ error }: { error: string | null }) {
    if (!error) return null;
    return (
        <div className="mx-6 mb-4 flex items-center gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-2">
            <AlertCircle size={13} className="text-destructive" />
            <span className="font-mono text-[11px] text-destructive">{error}</span>
        </div>
    );
}

// =============================================================================
// Generic namespace panel wrapper
// =============================================================================

function NamespacePanel({
    namespace,
    title,
    subtitle,
    saveDisabled,
    children,
}: {
    namespace: string;
    title: string;
    subtitle?: string;
    saveDisabled?: boolean;
    children: (ctx: {
        val: (key: string) => unknown;
        set: (key: string, value: unknown) => void;
    }) => ReactNode;
}) {
    const { values, loading, saving, error, setSetting, saveAll, reload } =
        useSettings(namespace);
    const [savedFlash, setSavedFlash] = useState(false);

    const handleSave = useCallback(async () => {
        await saveAll();
        setSavedFlash(true);
        setTimeout(() => setSavedFlash(false), 2500);
    }, [saveAll]);

    const val = (key: string) => values[`${namespace}.${key}`];
    const set = (key: string, value: unknown) =>
        setSetting(`${namespace}.${key}`, value);

    return (
        <div className="flex flex-1 flex-col overflow-hidden">
            <PanelHeader
                title={title}
                subtitle={subtitle}
                saving={saving}
                loading={loading}
                saveDisabled={saveDisabled}
                onSave={() => void handleSave()}
                onReload={() => void reload()}
            />
            <div className="flex-1 overflow-y-auto px-6 py-5">
                {loading && (
                    <div className="flex items-center gap-2">
                        <Loader2 size={15} className="animate-spin text-muted-foreground" />
                        <span className="font-mono text-[11px] text-muted-foreground">
                            Loading…
                        </span>
                    </div>
                )}
                <ErrorBanner error={error} />
                <SavedBanner show={savedFlash} />
                {!loading && (
                    <div className="space-y-6">{children({ val, set })}</div>
                )}
            </div>
        </div>
    );
}

// =============================================================================
// Individual panels
// =============================================================================

// --- LLM Providers ----------------------------------------------------------

function ProviderPanel() {
    const { values, settings, loading, saving, error, setSetting, saveAll, reload } =
        useSettings("provider");
    const [savedFlash, setSavedFlash] = useState(false);

    const providerEntries = Object.values(settings).sort((a, b) =>
        a.key.localeCompare(b.key)
    );

    const handleSave = useCallback(async () => {
        await saveAll();
        setSavedFlash(true);
        setTimeout(() => setSavedFlash(false), 2500);
    }, [saveAll]);

    return (
        <div className="flex flex-1 flex-col overflow-hidden">
            <PanelHeader
                title="LLM Providers"
                subtitle={`${providerEntries.length} provider(s) configured`}
                saving={saving}
                loading={loading}
                onSave={() => void handleSave()}
                onReload={() => void reload()}
            />
            <div className="flex-1 overflow-y-auto px-6 py-5 space-y-4">
                {loading && (
                    <div className="flex items-center gap-2">
                        <Loader2 size={15} className="animate-spin text-muted-foreground" />
                        <span className="font-mono text-[11px] text-muted-foreground">Loading…</span>
                    </div>
                )}
                <ErrorBanner error={error} />
                <SavedBanner show={savedFlash} />
                {!loading && providerEntries.length === 0 && (
                    <p className="font-mono text-[11px] text-muted-foreground">
                        No providers configured. Add providers in your config file to manage them here.
                    </p>
                )}
                {providerEntries.map((s) => {
                    const data = (values[s.key] ?? {}) as Record<string, unknown>;
                    const leafKey = s.key; // full key like provider.openai
                    const setField = (field: string, value: unknown) => {
                        const updated = { ...data, [field]: value };
                        setSetting(leafKey, updated);
                    };
                    const enabled = data.enabled !== false;
                    return (
                        <div
                            key={s.key}
                            className={cn(
                                "rounded-xl border p-4 space-y-4",
                                enabled ? "border-border bg-card" : "border-border/50 bg-muted/30 opacity-60"
                            )}
                        >
                            <div className="flex items-center justify-between">
                                <div>
                                    <p className="font-display text-[14px] font-semibold text-foreground">
                                        {(data.display_name as string) ?? s.name}
                                    </p>
                                    <p className="font-mono text-[10px] text-muted-foreground">
                                        {s.key}
                                    </p>
                                </div>
                                <Toggle
                                    value={enabled}
                                    onChange={(v) => setField("enabled", v)}
                                />
                            </div>
                            <div className="grid grid-cols-2 gap-3">
                                <Field label="Base URL">
                                    <Input
                                        value={(data.base_url as string) ?? ""}
                                        onChange={(e) => setField("base_url", e.target.value)}
                                        placeholder="https://api.example.com/v1"
                                        className="font-mono text-[12px]"
                                    />
                                </Field>
                                <Field label="Protocol">
                                    <SettingSelect
                                        value={(data.protocol as string) ?? "auto"}
                                        options={[
                                            { value: "auto", label: "Auto" },
                                            { value: "chat", label: "Chat Completions" },
                                            { value: "responses", label: "Responses API" },
                                        ]}
                                        onChange={(v) => setField("protocol", v)}
                                    />
                                </Field>
                                <Field label="API Key" hint="Masked for security">
                                    <MaskedInput
                                        value={(data.api_key as string) ?? ""}
                                        onChange={(v) => setField("api_key", v)}
                                        placeholder="sk-..."
                                    />
                                </Field>
                                <Field label="Default Model">
                                    <Input
                                        value={(data.default_model as string) ?? ""}
                                        onChange={(e) => setField("default_model", e.target.value)}
                                        placeholder="gpt-5.2"
                                        className="font-mono text-[12px]"
                                    />
                                </Field>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

// --- Vision -----------------------------------------------------------------

function VisionPanel() {
    return (
        <NamespacePanel namespace="vision" title="Vision Configuration">
            {({ val, set }) => (
                <>
                    <Field label="Vision Model Override" hint="Leave blank to use the active model's capabilities">
                        <Input
                            value={(val("model") as string) ?? ""}
                            onChange={(e) => set("model", e.target.value || null)}
                            placeholder="gpt-5.2 (inherits active model)"
                            className="font-mono text-[12px]"
                        />
                    </Field>
                    <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                        <div>
                            <p className="font-mono text-[12px] font-medium text-foreground">
                                Auto-detect Vision Capability
                            </p>
                            <p className="font-mono text-[10px] text-muted-foreground">
                                Automatically probe model capabilities on first use
                            </p>
                        </div>
                        <Toggle
                            value={(val("auto_detect") as boolean) ?? true}
                            onChange={(v) => set("auto_detect", v)}
                        />
                    </div>
                </>
            )}
        </NamespacePanel>
    );
}

// --- Context Management -----------------------------------------------------

const CONTEXT_STRATEGIES = [
    { value: "sliding_window", label: "Sliding Window" },
    { value: "keep_first_last", label: "Keep First & Last" },
    { value: "progressive_summarization", label: "Progressive Summarization" },
    { value: "hierarchical_memory", label: "Hierarchical Memory" },
    { value: "none", label: "None (no management)" },
];

function ContextManagementPanel() {
    return (
        <NamespacePanel
            namespace="context_management"
            title="Context Management"
            subtitle="Global defaults — agents can override individually"
        >
            {({ val, set }) => (
                <>
                    <Field
                        label="Strategy"
                        hint="How to handle conversations that approach the token limit"
                    >
                        <SettingSelect
                            value={(val("strategy") as string) ?? "sliding_window"}
                            options={CONTEXT_STRATEGIES}
                            onChange={(v) => set("strategy", v)}
                        />
                    </Field>
                    <div className="grid grid-cols-2 gap-4">
                        <Field
                            label="Max Tokens"
                            hint="Leave blank to use model's context window"
                        >
                            <Input
                                type="number"
                                value={(val("max_tokens") as number) ?? ""}
                                onChange={(e) =>
                                    set("max_tokens", e.target.value ? parseInt(e.target.value) : null)
                                }
                                placeholder="Blank = model limit"
                                min={512}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                        <Field label="Trigger Threshold" hint="0.1 – 1.0 (e.g. 0.85 = 85% full)">
                            <Input
                                type="number"
                                step={0.05}
                                min={0.1}
                                max={1.0}
                                value={(val("trigger_threshold") as number) ?? 0.85}
                                onChange={(e) => set("trigger_threshold", parseFloat(e.target.value))}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                        <Field label="Max Messages (sliding window)" hint="Maximum number of messages to keep">
                            <Input
                                type="number"
                                value={(val("max_messages") as number) ?? ""}
                                onChange={(e) =>
                                    set("max_messages", e.target.value ? parseInt(e.target.value) : null)
                                }
                                placeholder="Unlimited"
                                min={1}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                        <Field label="Summary Budget (tokens)" hint="Token budget for generated summaries">
                            <Input
                                type="number"
                                value={(val("summary_budget") as number) ?? 1000}
                                onChange={(e) => set("summary_budget", parseInt(e.target.value))}
                                min={100}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                    </div>
                    <Field
                        label="Summarization Model Override"
                        hint="Override which model performs summarization (blank = use active model)"
                    >
                        <Input
                            value={(val("summarization_model") as string) ?? ""}
                            onChange={(e) => set("summarization_model", e.target.value || null)}
                            placeholder="Inherits active model"
                            className="font-mono text-[12px]"
                        />
                    </Field>
                </>
            )}
        </NamespacePanel>
    );
}

// --- RAG & Chunking ---------------------------------------------------------

const CHUNKING_STRATEGIES = [
    { value: "fixed_size", label: "Fixed Size (chars)" },
    { value: "token", label: "Token-based" },
    { value: "recursive", label: "Recursive (semantic boundaries)" },
    { value: "sentence", label: "Sentence" },
    { value: "document", label: "Document (no chunking)" },
    { value: "semantic", label: "Semantic (embedding similarity)" },
    { value: "agentic", label: "Agentic (LLM-guided)" },
];

function RagPanel() {
    return (
        <NamespacePanel
            namespace="rag"
            title="RAG & Chunking"
            subtitle="Retrieval-augmented generation and document chunking defaults"
        >
            {({ val, set }) => {
                const strategy = (val("chunking_strategy") as string) ?? "recursive";
                return (
                    <>
                        <Field label="Chunking Strategy">
                            <SettingSelect
                                value={strategy}
                                options={CHUNKING_STRATEGIES}
                                onChange={(v) => set("chunking_strategy", v)}
                            />
                        </Field>
                        <div className="grid grid-cols-2 gap-4">
                            {(strategy === "fixed_size" || strategy === "recursive") && (
                                <Field label="Chunk Size (chars)">
                                    <Input
                                        type="number"
                                        value={(val("chunk_size") as number) ?? 1024}
                                        onChange={(e) => set("chunk_size", parseInt(e.target.value))}
                                        min={64}
                                        className="font-mono text-[12px]"
                                    />
                                </Field>
                            )}
                            {strategy === "token" && (
                                <Field label="Chunk Size (tokens)">
                                    <Input
                                        type="number"
                                        value={(val("chunk_tokens") as number) ?? 256}
                                        onChange={(e) => set("chunk_tokens", parseInt(e.target.value))}
                                        min={16}
                                        className="font-mono text-[12px]"
                                    />
                                </Field>
                            )}
                            {strategy === "semantic" && (
                                <Field label="Semantic Merge Threshold" hint="0.0 – 1.0 (higher = merge less)">
                                    <Input
                                        type="number"
                                        step={0.05}
                                        min={0.0}
                                        max={1.0}
                                        value={(val("semantic_threshold") as number) ?? 0.75}
                                        onChange={(e) => set("semantic_threshold", parseFloat(e.target.value))}
                                        className="font-mono text-[12px]"
                                    />
                                </Field>
                            )}
                        </div>
                        <div className="rounded-lg border border-border bg-card p-4 space-y-3">
                            <p className="font-mono text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
                                Embedding
                            </p>
                            <div className="grid grid-cols-2 gap-4">
                                <Field label="Embedding Provider ID">
                                    <Input
                                        value={(val("embedding_provider") as string) ?? ""}
                                        onChange={(e) => set("embedding_provider", e.target.value)}
                                        placeholder="openai"
                                        className="font-mono text-[12px]"
                                    />
                                </Field>
                                <Field label="Embedding Model ID">
                                    <Input
                                        value={(val("embedding_model") as string) ?? ""}
                                        onChange={(e) => set("embedding_model", e.target.value)}
                                        placeholder="text-embedding-3-small"
                                        className="font-mono text-[12px]"
                                    />
                                </Field>
                            </div>
                        </div>
                    </>
                );
            }}
        </NamespacePanel>
    );
}

// --- Knowledge Bases --------------------------------------------------------

function KnowledgeBasesPanel() {
    return (
        <NamespacePanel
            namespace="knowledge_bases"
            title="Knowledge Bases"
            subtitle="Default embedding and chunking configuration for all knowledge bases"
        >
            {({ val, set }) => {
                const defaultKb = (val("default") as Record<string, unknown>) ?? {};
                const setDefault = (field: string, value: unknown) =>
                    set("default", { ...defaultKb, [field]: value });
                return (
                    <>
                        <div className="grid grid-cols-2 gap-4">
                            <Field label="Default Embedding Provider">
                                <Input
                                    value={(defaultKb.embedding_provider as string) ?? ""}
                                    onChange={(e) => setDefault("embedding_provider", e.target.value)}
                                    placeholder="openai"
                                    className="font-mono text-[12px]"
                                />
                            </Field>
                            <Field label="Default Embedding Model">
                                <Input
                                    value={(defaultKb.embedding_model as string) ?? ""}
                                    onChange={(e) => setDefault("embedding_model", e.target.value)}
                                    placeholder="text-embedding-3-small"
                                    className="font-mono text-[12px]"
                                />
                            </Field>
                            <Field label="Chunking Strategy">
                                <Input
                                    value={(defaultKb.chunking_strategy as string) ?? ""}
                                    onChange={(e) => setDefault("chunking_strategy", e.target.value)}
                                    placeholder="recursive"
                                    className="font-mono text-[12px]"
                                />
                            </Field>
                            <Field label="Chunk Size">
                                <Input
                                    type="number"
                                    value={(defaultKb.chunk_size as number) ?? ""}
                                    onChange={(e) =>
                                        setDefault("chunk_size", e.target.value ? parseInt(e.target.value) : null)
                                    }
                                    placeholder="1024"
                                    className="font-mono text-[12px]"
                                />
                            </Field>
                        </div>
                    </>
                );
            }}
        </NamespacePanel>
    );
}

// --- File Processing --------------------------------------------------------

const FILE_PROC_PROVIDERS = [
    { value: "auto", label: "Auto" },
    { value: "unstructured", label: "Unstructured" },
    { value: "mistral", label: "Mistral OCR" },
    { value: "kreuzberg", label: "Kreuzberg" },
];

function FileProcessingPanel() {
    return (
        <NamespacePanel namespace="file_processing" title="File Processing">
            {({ val, set }) => (
                <>
                    <Field label="Default Provider">
                        <SettingSelect
                            value={(val("provider") as string) ?? "auto"}
                            options={FILE_PROC_PROVIDERS}
                            onChange={(v) => set("provider", v)}
                        />
                    </Field>
                    <Field label="Upload Directory">
                        <Input
                            value={(val("upload_dir") as string) ?? ""}
                            onChange={(e) => set("upload_dir", e.target.value)}
                            placeholder="/tmp/uploads"
                            className="font-mono text-[12px]"
                        />
                    </Field>
                    <div className="grid grid-cols-2 gap-4">
                        <Field label="Max Files Per Prompt">
                            <Input
                                type="number"
                                min={1}
                                value={(val("max_files_per_prompt") as number) ?? ""}
                                onChange={(e) => set("max_files_per_prompt", parseInt(e.target.value))}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                        <Field label="Max File Size (bytes)">
                            <Input
                                type="number"
                                min={0}
                                value={(val("max_file_size") as number) ?? ""}
                                onChange={(e) => set("max_file_size", parseInt(e.target.value))}
                                placeholder="10485760 (10MB)"
                                className="font-mono text-[12px]"
                            />
                        </Field>
                        <Field label="Max Total Size (bytes)">
                            <Input
                                type="number"
                                min={0}
                                value={(val("max_total_size") as number) ?? ""}
                                onChange={(e) => set("max_total_size", parseInt(e.target.value))}
                                placeholder="52428800 (50MB)"
                                className="font-mono text-[12px]"
                            />
                        </Field>
                    </div>
                </>
            )}
        </NamespacePanel>
    );
}

// --- Unstructured -----------------------------------------------------------

function UnstructuredPanel() {
    return (
        <NamespacePanel namespace="unstructured" title="Unstructured API">
            {({ val, set }) => (
                <>
                    <Field label="API URL">
                        <Input
                            value={(val("api_url") as string) ?? ""}
                            onChange={(e) => set("api_url", e.target.value)}
                            placeholder="https://api.unstructuredapp.io"
                            className="font-mono text-[12px]"
                        />
                    </Field>
                    <Field label="API Key" hint="Masked for security">
                        <MaskedInput
                            value={(val("api_key") as string) ?? ""}
                            onChange={(v) => set("api_key", v)}
                        />
                    </Field>
                </>
            )}
        </NamespacePanel>
    );
}

// --- Mistral OCR ------------------------------------------------------------

function MistralOcrPanel() {
    return (
        <NamespacePanel namespace="mistral_ocr" title="Mistral OCR">
            {({ val, set }) => (
                <>
                    <Field label="API Key" hint="Masked for security">
                        <MaskedInput
                            value={(val("api_key") as string) ?? ""}
                            onChange={(v) => set("api_key", v)}
                        />
                    </Field>
                    <Field label="OCR Model">
                        <Input
                            value={(val("ocr_model") as string) ?? ""}
                            onChange={(e) => set("ocr_model", e.target.value)}
                            placeholder="mistral-ocr-latest"
                            className="font-mono text-[12px]"
                        />
                    </Field>
                </>
            )}
        </NamespacePanel>
    );
}

// --- Kreuzberg --------------------------------------------------------------

function KreuzbergPanel() {
    return (
        <NamespacePanel namespace="kreuzberg" title="Kreuzberg OCR">
            {({ val, set }) => (
                <>
                    <div className="space-y-3">
                        {[
                            { key: "ocr_enabled", label: "OCR Enabled", hint: "Enable document OCR processing" },
                            { key: "force_ocr", label: "Force OCR", hint: "Always OCR even if text layer exists" },
                        ].map(({ key, label, hint }) => (
                            <div key={key} className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                                <div>
                                    <p className="font-mono text-[12px] font-medium text-foreground">{label}</p>
                                    <p className="font-mono text-[10px] text-muted-foreground">{hint}</p>
                                </div>
                                <Toggle
                                    value={(val(key) as boolean) ?? (key === "ocr_enabled")}
                                    onChange={(v) => set(key, v)}
                                />
                            </div>
                        ))}
                    </div>
                    <Field label="OCR Backend">
                        <Input
                            value={(val("ocr_backend") as string) ?? ""}
                            onChange={(e) => set("ocr_backend", e.target.value)}
                            placeholder="tesseract"
                            className="font-mono text-[12px]"
                        />
                    </Field>
                    <Field label="Output Format">
                        <SettingSelect
                            value={(val("output_format") as string) ?? "markdown"}
                            options={[
                                { value: "markdown", label: "Markdown" },
                                { value: "plain", label: "Plain Text" },
                                { value: "html", label: "HTML" },
                            ]}
                            onChange={(v) => set("output_format", v)}
                        />
                    </Field>
                </>
            )}
        </NamespacePanel>
    );
}

// --- Resilience -------------------------------------------------------------

const RESILIENCE_RECOMMENDED_DEFAULTS = {
    rate_limit_enabled: true,
    requests_per_second: 10,
    burst_size: 20,
    request_timeout_ms: 30000,
    stream_start_timeout_ms: 15000,
    retries_enabled: true,
    retry_max_attempts: 3,
    retry_base_delay_ms: 1000,
    retry_backoff_multiplier: 2,
    retry_max_delay_ms: 10000,
    retry_jitter_mode: "full",
    retry_respect_retry_after: true,
    retryable_http_statuses: [408, 425, 429, 500, 502, 503, 504],
    retryable_transport_errors: true,
    retry_budget_ms: 20000,
} as const;

function ResiliencePanel() {
    const { values, loading, saving, error, setSetting, saveAll, reload } = useSettings("resilience");
    const [savedFlash, setSavedFlash] = useState(false);
    const [showAdvanced, setShowAdvanced] = useState(false);
    const [statusListInput, setStatusListInput] = useState("");

    const valueFor = (key: string): unknown => values[`resilience.${key}`];
    const setField = (key: string, value: unknown) => setSetting(`resilience.${key}`, value);

    useEffect(() => {
        const statuses = valueFor("retryable_http_statuses");
        if (!Array.isArray(statuses)) return;
        setStatusListInput(statuses.join(", "));
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [values["resilience.retryable_http_statuses"]]);

    const parsedStatuses = statusListInput
        .split(",")
        .map((item) => item.trim())
        .filter(Boolean);
    const hasInvalidStatusToken = parsedStatuses.some((item) => {
        const n = Number(item);
        return !Number.isInteger(n) || n < 100 || n > 599;
    });
    const statusNumbers = parsedStatuses
        .map((item) => Number(item))
        .filter((n) => Number.isInteger(n) && n >= 100 && n <= 599);

    const validationErrors: Record<string, string> = {};
    const reqPerSec = Number(valueFor("requests_per_second"));
    if (!Number.isFinite(reqPerSec) || reqPerSec < 0.1) {
        validationErrors.requests_per_second = "Must be at least 0.1 requests/sec.";
    }
    const burstSize = Number(valueFor("burst_size"));
    if (!Number.isFinite(burstSize) || burstSize < 1) {
        validationErrors.burst_size = "Burst size must be at least 1.";
    }
    const requestTimeoutMs = Number(valueFor("request_timeout_ms"));
    if (!Number.isFinite(requestTimeoutMs) || requestTimeoutMs < 1000) {
        validationErrors.request_timeout_ms = "Request timeout must be at least 1000ms.";
    }
    const streamStartTimeoutMs = Number(valueFor("stream_start_timeout_ms"));
    if (!Number.isFinite(streamStartTimeoutMs) || streamStartTimeoutMs < 1000) {
        validationErrors.stream_start_timeout_ms = "Stream start timeout must be at least 1000ms.";
    }
    const retryMaxAttempts = Number(valueFor("retry_max_attempts"));
    if (!Number.isFinite(retryMaxAttempts) || retryMaxAttempts < 0 || retryMaxAttempts > 10) {
        validationErrors.retry_max_attempts = "Retry attempts must be between 0 and 10.";
    }
    const retryBaseDelayMs = Number(valueFor("retry_base_delay_ms"));
    if (!Number.isFinite(retryBaseDelayMs) || retryBaseDelayMs < 100) {
        validationErrors.retry_base_delay_ms = "Base delay must be at least 100ms.";
    }
    const retryBackoffMultiplier = Number(valueFor("retry_backoff_multiplier"));
    if (
        !Number.isFinite(retryBackoffMultiplier)
        || retryBackoffMultiplier < 1.1
        || retryBackoffMultiplier > 5.0
    ) {
        validationErrors.retry_backoff_multiplier = "Backoff multiplier must be between 1.1 and 5.0.";
    }
    const retryMaxDelayMs = Number(valueFor("retry_max_delay_ms"));
    if (!Number.isFinite(retryMaxDelayMs) || retryMaxDelayMs < 100) {
        validationErrors.retry_max_delay_ms = "Max delay must be at least 100ms.";
    }
    const retryBudgetMs = Number(valueFor("retry_budget_ms"));
    if (!Number.isFinite(retryBudgetMs) || retryBudgetMs < 0) {
        validationErrors.retry_budget_ms = "Retry budget must be 0 or greater.";
    }
    if (hasInvalidStatusToken || statusNumbers.length === 0) {
        validationErrors.retryable_http_statuses = "Use comma-separated HTTP codes between 100 and 599.";
    }

    const hasErrors = Object.keys(validationErrors).length > 0;

    const handleSave = useCallback(async () => {
        if (hasErrors) return;
        await saveAll();
        setSavedFlash(true);
        setTimeout(() => setSavedFlash(false), 2500);
    }, [hasErrors, saveAll]);

    const resetRecommendedDefaults = useCallback(() => {
        Object.entries(RESILIENCE_RECOMMENDED_DEFAULTS).forEach(([key, value]) => {
            setField(key, value);
        });
        setStatusListInput(RESILIENCE_RECOMMENDED_DEFAULTS.retryable_http_statuses.join(", "));
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    const renderError = (key: string) => validationErrors[key]
        ? <p className="font-mono text-[10px] text-destructive">{validationErrors[key]}</p>
        : null;

    return (
        <div className="flex flex-1 flex-col overflow-hidden">
            <PanelHeader
                title="Resilience"
                subtitle="Configure global rate limits, timeout budgets, and retry behavior."
                saving={saving}
                loading={loading}
                saveDisabled={hasErrors}
                onSave={() => void handleSave()}
                onReload={() => void reload()}
            />
            <div className="flex-1 overflow-y-auto px-6 py-5 space-y-6">
                {loading && (
                    <div className="flex items-center gap-2">
                        <Loader2 size={15} className="animate-spin text-muted-foreground" />
                        <span className="font-mono text-[11px] text-muted-foreground">Loading…</span>
                    </div>
                )}
                <ErrorBanner error={error} />
                <SavedBanner show={savedFlash} />
                {!loading && (
                    <>
                        <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                            <div>
                                <p className="font-mono text-[12px] font-medium text-foreground">Recommended defaults</p>
                                <p className="font-mono text-[10px] text-muted-foreground">Reset all resilience controls to production-safe defaults.</p>
                            </div>
                            <Button type="button" variant="outline" size="sm" onClick={resetRecommendedDefaults}>
                                Reset Defaults
                            </Button>
                        </div>

                        <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50">Rate limiting</p>
                        <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                            <div>
                                <p className="font-mono text-[12px] font-medium text-foreground">Rate Limiting</p>
                                <p className="font-mono text-[10px] text-muted-foreground">Throttle inbound API traffic to reduce overload bursts.</p>
                            </div>
                            <Toggle
                                value={(valueFor("rate_limit_enabled") as boolean) ?? true}
                                onChange={(v) => setField("rate_limit_enabled", v)}
                            />
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                            <Field label="Requests Per Second" hint="Lower value = stricter global throttling.">
                                <Input
                                    type="number"
                                    step={0.1}
                                    min={0.1}
                                    value={(valueFor("requests_per_second") as number) ?? ""}
                                    onChange={(e) => setField("requests_per_second", Number(e.target.value))}
                                    className="font-mono text-[12px]"
                                />
                                {renderError("requests_per_second")}
                            </Field>
                            <Field label="Burst Size" hint="How many requests can pass in a short spike.">
                                <Input
                                    type="number"
                                    min={1}
                                    value={(valueFor("burst_size") as number) ?? ""}
                                    onChange={(e) => setField("burst_size", Number(e.target.value))}
                                    className="font-mono text-[12px]"
                                />
                                {renderError("burst_size")}
                            </Field>
                        </div>

                        <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50">Timeouts</p>
                        <div className="grid grid-cols-2 gap-4">
                            <Field label="Request Timeout (ms)" hint="Upper bound for non-stream response wait time.">
                                <Input
                                    type="number"
                                    min={1000}
                                    value={(valueFor("request_timeout_ms") as number) ?? ""}
                                    onChange={(e) => setField("request_timeout_ms", Number(e.target.value))}
                                    className="font-mono text-[12px]"
                                />
                                {renderError("request_timeout_ms")}
                            </Field>
                            <Field label="Stream Start Timeout (ms)" hint="How long to wait for first stream chunk before retry/fail.">
                                <Input
                                    type="number"
                                    min={1000}
                                    value={(valueFor("stream_start_timeout_ms") as number) ?? ""}
                                    onChange={(e) => setField("stream_start_timeout_ms", Number(e.target.value))}
                                    className="font-mono text-[12px]"
                                />
                                {renderError("stream_start_timeout_ms")}
                            </Field>
                        </div>

                        <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50">Retries</p>
                        <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                            <div>
                                <p className="font-mono text-[12px] font-medium text-foreground">Retries Enabled</p>
                                <p className="font-mono text-[10px] text-muted-foreground">Retry transient failures with exponential backoff.</p>
                            </div>
                            <Toggle
                                value={(valueFor("retries_enabled") as boolean) ?? true}
                                onChange={(v) => setField("retries_enabled", v)}
                            />
                        </div>
                        <div className="grid grid-cols-3 gap-4">
                            <Field label="Max Attempts" hint="Total attempts including the first request.">
                                <Input
                                    type="number"
                                    min={0}
                                    max={10}
                                    value={(valueFor("retry_max_attempts") as number) ?? ""}
                                    onChange={(e) => setField("retry_max_attempts", Number(e.target.value))}
                                    className="font-mono text-[12px]"
                                />
                                {renderError("retry_max_attempts")}
                            </Field>
                            <Field label="Base Delay (ms)" hint="Initial retry delay before backoff multiplier.">
                                <Input
                                    type="number"
                                    min={100}
                                    value={(valueFor("retry_base_delay_ms") as number) ?? ""}
                                    onChange={(e) => setField("retry_base_delay_ms", Number(e.target.value))}
                                    className="font-mono text-[12px]"
                                />
                                {renderError("retry_base_delay_ms")}
                            </Field>
                            <Field label="Max Delay (ms)" hint="Hard cap for retry wait interval.">
                                <Input
                                    type="number"
                                    min={100}
                                    value={(valueFor("retry_max_delay_ms") as number) ?? ""}
                                    onChange={(e) => setField("retry_max_delay_ms", Number(e.target.value))}
                                    className="font-mono text-[12px]"
                                />
                                {renderError("retry_max_delay_ms")}
                            </Field>
                        </div>

                        <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="justify-start px-0 font-mono text-[11px] text-muted-foreground hover:text-foreground"
                            onClick={() => setShowAdvanced((v) => !v)}
                        >
                            {showAdvanced ? "Hide advanced retry controls" : "Show advanced retry controls"}
                        </Button>

                        {showAdvanced && (
                            <div className="space-y-4 rounded-lg border border-border bg-card p-4">
                                <div className="grid grid-cols-2 gap-4">
                                    <Field label="Backoff Multiplier" hint="Growth factor for each retry delay step.">
                                        <Input
                                            type="number"
                                            step={0.1}
                                            min={1.1}
                                            max={5}
                                            value={(valueFor("retry_backoff_multiplier") as number) ?? ""}
                                            onChange={(e) => setField("retry_backoff_multiplier", Number(e.target.value))}
                                            className="font-mono text-[12px]"
                                        />
                                        {renderError("retry_backoff_multiplier")}
                                    </Field>
                                    <Field label="Retry Budget (ms)" hint="Maximum total waiting time spent on retries.">
                                        <Input
                                            type="number"
                                            min={0}
                                            value={(valueFor("retry_budget_ms") as number) ?? ""}
                                            onChange={(e) => setField("retry_budget_ms", Number(e.target.value))}
                                            className="font-mono text-[12px]"
                                        />
                                        {renderError("retry_budget_ms")}
                                    </Field>
                                    <Field label="Jitter Mode" hint="Jitter spreads retries across clients to avoid synchronized spikes.">
                                        <SettingSelect
                                            value={(valueFor("retry_jitter_mode") as string) ?? "full"}
                                            options={[
                                                { value: "none", label: "None" },
                                                { value: "full", label: "Full" },
                                                { value: "equal", label: "Equal" },
                                                { value: "decorrelated", label: "Decorrelated" },
                                            ]}
                                            onChange={(v) => setField("retry_jitter_mode", v)}
                                        />
                                    </Field>
                                    <div className="space-y-3">
                                        <div className="flex items-center justify-between rounded-md border border-border bg-muted/20 px-3 py-2">
                                            <span className="font-mono text-[11px] text-foreground">Respect Retry-After</span>
                                            <Toggle
                                                value={(valueFor("retry_respect_retry_after") as boolean) ?? true}
                                                onChange={(v) => setField("retry_respect_retry_after", v)}
                                            />
                                        </div>
                                        <div className="flex items-center justify-between rounded-md border border-border bg-muted/20 px-3 py-2">
                                            <span className="font-mono text-[11px] text-foreground">Retry Transport Errors</span>
                                            <Toggle
                                                value={(valueFor("retryable_transport_errors") as boolean) ?? true}
                                                onChange={(v) => setField("retryable_transport_errors", v)}
                                            />
                                        </div>
                                    </div>
                                </div>
                                <Field label="Retryable HTTP Status Codes" hint="Comma-separated status codes (100-599).">
                                    <Input
                                        value={statusListInput}
                                        onChange={(e) => {
                                            const raw = e.target.value;
                                            setStatusListInput(raw);
                                            const next = raw
                                                .split(",")
                                                .map((item) => Number(item.trim()))
                                                .filter((n) => Number.isInteger(n) && n >= 100 && n <= 599);
                                            if (next.length > 0) setField("retryable_http_statuses", next);
                                        }}
                                        placeholder="408, 425, 429, 500, 502, 503, 504"
                                        className="font-mono text-[12px]"
                                    />
                                    {renderError("retryable_http_statuses")}
                                </Field>
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
}

function buildGlobalResiliencePreview(
    values: Record<string, unknown>,
): Record<string, unknown> {
    const global = { ...RESILIENCE_RECOMMENDED_DEFAULTS } as Record<string, unknown>;
    Object.keys(global).forEach((key) => {
        const candidate = values[`resilience.${key}`];
        if (candidate !== undefined) {
            global[key] = candidate;
        }
    });
    return global;
}

function mergeAgentResiliencePreview(
    global: Record<string, unknown>,
    override: Record<string, unknown>,
): Record<string, unknown> {
    if ((override.mode as string) !== "override") {
        return global;
    }
    const merged = { ...global };
    Object.entries(override).forEach(([k, v]) => {
        if (k === "mode") return;
        if (v !== undefined && v !== null) {
            merged[k] = v;
        }
    });
    return merged;
}

// --- Intent Classifier ------------------------------------------------------

const IC_BACKENDS = [
    { value: "rules", label: "Rules" },
    { value: "tfidf", label: "TF-IDF" },
    { value: "wasm", label: "WASM Component" },
    { value: "hybrid", label: "Hybrid" },
    { value: "localembedding", label: "Local Embeddings" },
    { value: "llm", label: "LLM" },
];

function IntentClassifierPanel() {
    return (
        <NamespacePanel namespace="intent_classifier" title="Intent Classifier">
            {({ val, set }) => (
                <>
                    <Field label="Backend">
                        <SettingSelect
                            value={(val("backend") as string) ?? "rules"}
                            options={IC_BACKENDS}
                            onChange={(v) => set("backend", v)}
                        />
                    </Field>
                    <div className="grid grid-cols-2 gap-4">
                        <Field label="Top-K Results">
                            <Input
                                type="number"
                                min={1}
                                value={(val("topk") as number) ?? 3}
                                onChange={(e) => set("topk", parseInt(e.target.value))}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                        <Field label="Accept Threshold" hint="0.0 – 1.0">
                            <Input
                                type="number"
                                step={0.05}
                                min={0.0}
                                max={1.0}
                                value={(val("accept_threshold") as number) ?? 0.7}
                                onChange={(e) => set("accept_threshold", parseFloat(e.target.value))}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                        <Field label="Margin Threshold" hint="0.0 – 1.0 (confidence margin between top results)">
                            <Input
                                type="number"
                                step={0.05}
                                min={0.0}
                                max={1.0}
                                value={(val("margin_threshold") as number) ?? 0.1}
                                onChange={(e) => set("margin_threshold", parseFloat(e.target.value))}
                                className="font-mono text-[12px]"
                            />
                        </Field>
                    </div>
                    {(val("backend") as string) === "wasm" && (
                        <Field label="WASM Component Path">
                            <Input
                                value={(val("wasm_component_path") as string) ?? ""}
                                onChange={(e) => set("wasm_component_path", e.target.value || null)}
                                placeholder="/path/to/component.wasm"
                                className="font-mono text-[12px]"
                            />
                        </Field>
                    )}
                </>
            )}
        </NamespacePanel>
    );
}

// --- Governance -------------------------------------------------------------

const GOVERNANCE_ACTIONS = [
    { value: "execute_tool", label: "Execute Tool" },
    { value: "call_llm", label: "Call LLM" },
    { value: "spawn_agent", label: "Spawn Agent" },
    { value: "collaborate", label: "Collaborate (agent-to-agent)" },
    { value: "access_knowledge", label: "Access Knowledge Base" },
];

function GovernancePanel() {
    return (
        <NamespacePanel
            namespace="governance"
            title="Governance"
            subtitle="Cedar-policy based authorization defaults"
        >
            {({ val, set }) => {
                const mode = (val("default_mode") as string) ?? "permit_all";
                const allowed = (val("allowed_actions") as string[]) ?? [];
                const toggleAction = (action: string) => {
                    const next = allowed.includes(action)
                        ? allowed.filter((a) => a !== action)
                        : [...allowed, action];
                    set("allowed_actions", next);
                };
                return (
                    <>
                        <Field
                            label="Default Authorization Mode"
                            hint="Applied when no specific Cedar policy matches"
                        >
                            <SettingSelect
                                value={mode}
                                options={[
                                    { value: "permit_all", label: "Permit All (default allow)" },
                                    { value: "deny_all", label: "Deny All (default deny)" },
                                    { value: "custom", label: "Custom (policy files only)" },
                                ]}
                                onChange={(v) => set("default_mode", v)}
                            />
                        </Field>
                        {mode !== "permit_all" && (
                            <div className="space-y-2">
                                <Label className="font-mono text-[11px] font-medium text-muted-foreground uppercase tracking-wide">
                                    Globally Allowed Actions
                                </Label>
                                <div className="space-y-2">
                                    {GOVERNANCE_ACTIONS.map(({ value, label }) => (
                                        <label
                                            key={value}
                                            className="flex cursor-pointer items-center gap-3 rounded-lg border border-border bg-card px-4 py-2.5"
                                        >
                                            <input
                                                type="checkbox"
                                                checked={allowed.includes(value)}
                                                onChange={() => toggleAction(value)}
                                                className="accent-primary"
                                            />
                                            <span className="font-mono text-[12px] text-foreground">{label}</span>
                                            <span className="ml-auto font-mono text-[10px] text-muted-foreground">
                                                {value}
                                            </span>
                                        </label>
                                    ))}
                                </div>
                            </div>
                        )}
                        <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                            <div>
                                <p className="font-mono text-[12px] font-medium text-foreground">
                                    Hot Policy Reload
                                </p>
                                <p className="font-mono text-[10px] text-muted-foreground">
                                    Reload Cedar policy files without restarting the server
                                </p>
                            </div>
                            <Toggle
                                value={(val("policy_reload_enabled") as boolean) ?? true}
                                onChange={(v) => set("policy_reload_enabled", v)}
                            />
                        </div>
                    </>
                );
            }}
        </NamespacePanel>
    );
}

// --- Agent Config -----------------------------------------------------------

const AGENT_CONTEXT_OPTS = [
    { value: "inherit", label: "Inherit Global" },
    { value: "sliding_window", label: "Sliding Window" },
    { value: "keep_first_last", label: "Keep First & Last" },
    { value: "progressive_summarization", label: "Progressive Summarization" },
    { value: "none", label: "None" },
];

const AGENT_GOV_OPTS = [
    { value: "inherit", label: "Inherit Global" },
    { value: "permit_all", label: "Permit All" },
    { value: "deny_all", label: "Deny All" },
];

const AGENT_RESILIENCE_MODE_OPTS = [
    { value: "inherit", label: "Inherit Global" },
    { value: "override", label: "Override" },
];

function AgentConfigPanel() {
    const { values, settings, loading, saving, error, setSetting, saveAll, reload } =
        useSettings("agent_config");
    const { values: resilienceValues } = useSettings("resilience");
    const [savedFlash, setSavedFlash] = useState(false);

    const agentEntries = Object.values(settings).sort((a, b) => a.key.localeCompare(b.key));

    const handleSave = useCallback(async () => {
        await saveAll();
        setSavedFlash(true);
        setTimeout(() => setSavedFlash(false), 2500);
    }, [saveAll]);

    return (
        <div className="flex flex-1 flex-col overflow-hidden">
            <PanelHeader
                title="Agent Configuration"
                subtitle={`${agentEntries.length} agent(s) configured`}
                saving={saving}
                loading={loading}
                onSave={() => void handleSave()}
                onReload={() => void reload()}
            />
            <div className="flex-1 overflow-y-auto px-6 py-5 space-y-4">
                {loading && (
                    <div className="flex items-center gap-2">
                        <Loader2 size={15} className="animate-spin text-muted-foreground" />
                        <span className="font-mono text-[11px] text-muted-foreground">Loading…</span>
                    </div>
                )}
                <ErrorBanner error={error} />
                <SavedBanner show={savedFlash} />
                {agentEntries.map((s) => {
                    const data = (values[s.key] ?? {}) as Record<string, unknown>;
                    const agentId = s.key.replace("agent_config.", "");
                    const setField = (field: string, value: unknown) =>
                        setSetting(s.key, { ...data, [field]: value });
                    const resilienceOverride = (data.resilience as Record<string, unknown>) ?? { mode: "inherit" };
                    const setResilienceField = (field: string, value: unknown) =>
                        setField("resilience", { ...resilienceOverride, [field]: value });
                    const enabled = data.enabled !== false;
                    const resilienceMode = (resilienceOverride.mode as string) ?? "inherit";
                    const globalPreview = buildGlobalResiliencePreview(resilienceValues);
                    const effectivePreview = mergeAgentResiliencePreview(globalPreview, resilienceOverride);
                    return (
                        <div
                            key={s.key}
                            className={cn(
                                "rounded-xl border p-4 space-y-4",
                                enabled ? "border-border bg-card" : "border-border/50 bg-muted/30 opacity-60"
                            )}
                        >
                            <div className="flex items-center justify-between">
                                <div>
                                    <p className="font-display text-[14px] font-semibold text-foreground flex items-center gap-2">
                                        <Bot size={14} className="text-muted-foreground" />
                                        {s.name}
                                    </p>
                                    <p className="font-mono text-[10px] text-muted-foreground">{agentId}</p>
                                </div>
                                <Toggle
                                    value={enabled}
                                    onChange={(v) => setField("enabled", v)}
                                />
                            </div>
                            <div className="grid grid-cols-2 gap-3">
                                <Field label="Context Strategy">
                                    <SettingSelect
                                        value={(data.context_strategy as string) ?? "inherit"}
                                        options={AGENT_CONTEXT_OPTS}
                                        onChange={(v) => setField("context_strategy", v)}
                                    />
                                </Field>
                                <Field label="Governance Mode">
                                    <SettingSelect
                                        value={(data.governance_mode as string) ?? "inherit"}
                                        options={AGENT_GOV_OPTS}
                                        onChange={(v) => setField("governance_mode", v)}
                                    />
                                </Field>
                            </div>
                            <Field
                                label="Allowed Tools"
                                hint="Comma-separated tool names. Leave blank to allow all."
                            >
                                <Input
                                    value={
                                        Array.isArray(data.allowed_tools)
                                            ? (data.allowed_tools as string[]).join(", ")
                                            : ""
                                    }
                                    onChange={(e) =>
                                        setField(
                                            "allowed_tools",
                                            e.target.value
                                                ? e.target.value.split(",").map((t) => t.trim()).filter(Boolean)
                                                : []
                                        )
                                    }
                                    placeholder="web_search, code_exec, … (blank = all)"
                                    className="font-mono text-[12px]"
                                />
                            </Field>
                            <div className="space-y-3 rounded-lg border border-border/60 bg-muted/20 p-3">
                                <Field label="Resilience Policy">
                                    <SettingSelect
                                        value={resilienceMode}
                                        options={AGENT_RESILIENCE_MODE_OPTS}
                                        onChange={(v) => setResilienceField("mode", v)}
                                    />
                                </Field>
                                {resilienceMode === "override" && (
                                    <div className="grid grid-cols-2 gap-3">
                                        <Field label="Request Timeout (ms)">
                                            <Input
                                                type="number"
                                                min={1000}
                                                value={(resilienceOverride.request_timeout_ms as number) ?? ""}
                                                onChange={(e) => setResilienceField("request_timeout_ms", e.target.value ? Number(e.target.value) : null)}
                                                placeholder={`${globalPreview.request_timeout_ms ?? ""}`}
                                                className="font-mono text-[12px]"
                                            />
                                        </Field>
                                        <Field label="Retry Max Attempts">
                                            <Input
                                                type="number"
                                                min={0}
                                                max={10}
                                                value={(resilienceOverride.retry_max_attempts as number) ?? ""}
                                                onChange={(e) => setResilienceField("retry_max_attempts", e.target.value ? Number(e.target.value) : null)}
                                                placeholder={`${globalPreview.retry_max_attempts ?? ""}`}
                                                className="font-mono text-[12px]"
                                            />
                                        </Field>
                                        <Field label="Retry Base Delay (ms)">
                                            <Input
                                                type="number"
                                                min={100}
                                                value={(resilienceOverride.retry_base_delay_ms as number) ?? ""}
                                                onChange={(e) => setResilienceField("retry_base_delay_ms", e.target.value ? Number(e.target.value) : null)}
                                                placeholder={`${globalPreview.retry_base_delay_ms ?? ""}`}
                                                className="font-mono text-[12px]"
                                            />
                                        </Field>
                                        <Field label="Retry Max Delay (ms)">
                                            <Input
                                                type="number"
                                                min={100}
                                                value={(resilienceOverride.retry_max_delay_ms as number) ?? ""}
                                                onChange={(e) => setResilienceField("retry_max_delay_ms", e.target.value ? Number(e.target.value) : null)}
                                                placeholder={`${globalPreview.retry_max_delay_ms ?? ""}`}
                                                className="font-mono text-[12px]"
                                            />
                                        </Field>
                                    </div>
                                )}
                                <div className="space-y-1">
                                    <p className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
                                        Effective policy preview
                                    </p>
                                    <pre className="max-h-36 overflow-auto rounded-md border border-border/60 bg-background/80 p-2 font-mono text-[10px] text-muted-foreground">
                                        {JSON.stringify(effectivePreview, null, 2)}
                                    </pre>
                                </div>
                            </div>
                        </div>
                    );
                })}
                {!loading && agentEntries.length === 0 && (
                    <p className="font-mono text-[11px] text-muted-foreground">
                        No agents configured yet.
                    </p>
                )}
            </div>
        </div>
    );
}

// --- Skill Config -----------------------------------------------------------

function SkillConfigPanel() {
    const { values, settings, loading, saving, error, setSetting, saveAll, reload } =
        useSettings("skill_config");
    const [savedFlash, setSavedFlash] = useState(false);

    const skillEntries = Object.values(settings).sort((a, b) => a.key.localeCompare(b.key));

    const handleSave = useCallback(async () => {
        await saveAll();
        setSavedFlash(true);
        setTimeout(() => setSavedFlash(false), 2500);
    }, [saveAll]);

    return (
        <div className="flex flex-1 flex-col overflow-hidden">
            <PanelHeader
                title="Skill Configuration"
                subtitle={`${skillEntries.length} built-in skill(s)`}
                saving={saving}
                loading={loading}
                onSave={() => void handleSave()}
                onReload={() => void reload()}
            />
            <div className="flex-1 overflow-y-auto px-6 py-5 space-y-2">
                {loading && (
                    <div className="flex items-center gap-2">
                        <Loader2 size={15} className="animate-spin text-muted-foreground" />
                        <span className="font-mono text-[11px] text-muted-foreground">Loading…</span>
                    </div>
                )}
                <ErrorBanner error={error} />
                <SavedBanner show={savedFlash} />
                {skillEntries.map((s) => {
                    const data = (values[s.key] ?? {}) as Record<string, unknown>;
                    const skillId = s.key.replace("skill_config.", "");
                    const setField = (field: string, value: unknown) =>
                        setSetting(s.key, { ...data, [field]: value });
                    const enabled = data.enabled !== false;
                    return (
                        <div
                            key={s.key}
                            className={cn(
                                "flex items-start gap-4 rounded-lg border px-4 py-3 transition-colors",
                                enabled ? "border-border bg-card" : "border-border/40 bg-muted/20"
                            )}
                        >
                            <div className="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md bg-muted">
                                <Zap size={14} className={cn(enabled ? "text-primary" : "text-muted-foreground")} />
                            </div>
                            <div className="min-w-0 flex-1 space-y-1">
                                <p className={cn("font-mono text-[13px] font-medium", enabled ? "text-foreground" : "text-muted-foreground")}>
                                    {skillId}
                                </p>
                                <p className="font-mono text-[10px] text-muted-foreground">
                                    {(data.description as string) ?? ""}
                                </p>
                                <Input
                                    value={(data.description as string) ?? ""}
                                    onChange={(e) => setField("description", e.target.value)}
                                    placeholder="Override description…"
                                    className="mt-1.5 h-7 font-mono text-[11px]"
                                />
                            </div>
                            <Toggle
                                value={enabled}
                                onChange={(v) => setField("enabled", v)}
                            />
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

// =============================================================================
// Main Settings Page
// =============================================================================

// =============================================================================
// Memory panel
// =============================================================================

function MemoryPanel() {
    return (
        <NamespacePanel
            namespace="memory"
            title="Agent Memory"
            subtitle="Persistent memory across conversations: auto-capture, context injection, and scoped retrieval."
        >
            {({ val, set }) => (
                <>
                    <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50 mb-2">
                        Global Control
                    </p>
                    <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
                        <div>
                            <p className="font-mono text-[12px] font-medium text-foreground">Enable Memory System</p>
                            <p className="font-mono text-[10px] text-muted-foreground">Initialize the in-process vector memory store on startup.</p>
                        </div>
                        <Toggle value={(val("enabled") as boolean) ?? false} onChange={(v) => set("enabled", v)} />
                    </div>

                    <Field label="Activation Mode" hint="auto: capture+inject every turn. tool_only: LLM invokes memory tools. disabled: off.">
                        <SettingSelect
                            value={(val("activation_mode") as string) ?? "auto"}
                            options={[
                                { value: "auto", label: "auto (capture + inject)" },
                                { value: "tool_only", label: "tool_only (LLM-driven)" },
                                { value: "disabled", label: "disabled" },
                            ]}
                            onChange={(v) => set("activation_mode", v)}
                        />
                    </Field>

                    <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
                        Auto-Capture &amp; Injection
                    </p>
                    <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
                        <div>
                            <p className="font-mono text-[12px] font-medium text-foreground">Auto-Capture</p>
                            <p className="font-mono text-[10px] text-muted-foreground">Extract memories from each completed assistant turn.</p>
                        </div>
                        <Toggle value={(val("auto_capture") as boolean) ?? true} onChange={(v) => set("auto_capture", v)} />
                    </div>
                    <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
                        <div>
                            <p className="font-mono text-[12px] font-medium text-foreground">Context Injection</p>
                            <p className="font-mono text-[10px] text-muted-foreground">Inject relevant memories as a system prompt prefix before each LLM call.</p>
                        </div>
                        <Toggle value={(val("inject_context") as boolean) ?? true} onChange={(v) => set("inject_context", v)} />
                    </div>
                    <Field label="Max Context Tokens" hint="Maximum tokens to spend on injected memory context.">
                        <Input
                            type="number" min={100} max={32000}
                            value={(val("max_context_tokens") as number) ?? 4096}
                            onChange={(e) => set("max_context_tokens", Number(e.target.value))}
                            className="w-32 font-mono text-[12px]"
                        />
                    </Field>

                    <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
                        Retrieval Weights
                    </p>
                    <Field label="Vector (Semantic) Weight" hint="0.0–1.0. Higher = more semantic similarity.">
                        <Input
                            type="number" min={0} max={1} step={0.05}
                            value={(val("vector_weight") as number) ?? 0.7}
                            onChange={(e) => set("vector_weight", parseFloat(e.target.value))}
                            className="w-28 font-mono text-[12px]"
                        />
                    </Field>
                    <Field label="BM25 (Keyword) Weight" hint="0.0–1.0. Higher = more keyword match.">
                        <Input
                            type="number" min={0} max={1} step={0.05}
                            value={(val("bm25_weight") as number) ?? 0.3}
                            onChange={(e) => set("bm25_weight", parseFloat(e.target.value))}
                            className="w-28 font-mono text-[12px]"
                        />
                    </Field>

                    <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
                        Embedding Provider
                    </p>
                    <Field label="Provider">
                        <SettingSelect
                            value={(val("embedding_provider") as string) ?? "openai"}
                            options={[
                                { value: "openai", label: "OpenAI" },
                                { value: "cohere", label: "Cohere" },
                                { value: "local", label: "Local" },
                            ]}
                            onChange={(v) => set("embedding_provider", v)}
                        />
                    </Field>
                    <Field label="Embedding Model" hint="Provider-specific model name.">
                        <Input
                            value={(val("embedding_model") as string) ?? "text-embedding-3-small"}
                            onChange={(e) => set("embedding_model", e.target.value)}
                            placeholder="text-embedding-3-small"
                            className="w-64 font-mono text-[12px]"
                        />
                    </Field>

                    <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
                        Storage
                    </p>
                    <Field label="DB Path" hint="Filesystem path for the embedded SurrealDB memory store.">
                        <Input
                            value={(val("db_path") as string) ?? "data/memory"}
                            onChange={(e) => set("db_path", e.target.value)}
                            placeholder="data/memory"
                            className="w-64 font-mono text-[12px]"
                        />
                    </Field>

                    <p className="font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
                        MCP HTTP Endpoint
                    </p>
                    <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
                        <div>
                            <p className="font-mono text-[12px] font-medium text-foreground">Enable MCP Endpoint</p>
                            <p className="font-mono text-[10px] text-muted-foreground">Expose memory tools over HTTP as a Model Context Protocol server.</p>
                        </div>
                        <Toggle value={(val("mcp_http_enabled") as boolean) ?? false} onChange={(v) => set("mcp_http_enabled", v)} />
                    </div>
                    <Field label="MCP HTTP Path">
                        <Input
                            value={(val("mcp_http_path") as string) ?? "/mcp/memory"}
                            onChange={(e) => set("mcp_http_path", e.target.value)}
                            placeholder="/mcp/memory"
                            className="w-48 font-mono text-[12px]"
                        />
                    </Field>
                </>
            )}
        </NamespacePanel>
    );
}


const PANEL_MAP: Record<string, () => ReactNode> = {
    provider: () => <ProviderPanel />,
    vision: () => <VisionPanel />,
    context_management: () => <ContextManagementPanel />,
    rag: () => <RagPanel />,
    knowledge_bases: () => <KnowledgeBasesPanel />,
    memory: () => <MemoryPanel />,
    file_processing: () => <FileProcessingPanel />,
    unstructured: () => <UnstructuredPanel />,
    mistral_ocr: () => <MistralOcrPanel />,
    kreuzberg: () => <KreuzbergPanel />,
    resilience: () => <ResiliencePanel />,
    intent_classifier: () => <IntentClassifierPanel />,
    governance: () => <GovernancePanel />,
    agent_config: () => <AgentConfigPanel />,
    skill_config: () => <SkillConfigPanel />,
};

export const SettingsPage: FC = () => {
    const [active, setActive] = useState<string>("provider");
    const [types, setTypes] = useState<SettingsType[]>([]);

    // Fetch registered types to know which namespaces are available at runtime
    useEffect(() => {
        fetch("/api/uar/settings/types")
            .then((r) => r.json())
            .then((d) => setTypes(Array.isArray(d) ? d : []))
            .catch(() => {/* non-fatal */ });
    }, []);

    const availableKeys = new Set(types.map((t) => t.key));

    return (
        <div className="flex flex-1 overflow-hidden">
            {/* Sidebar */}
            <aside className="flex w-52 shrink-0 flex-col border-r border-border bg-card">
                <div className="border-b border-border px-4 py-3">
                    <p className="font-mono text-[10px] font-medium uppercase tracking-widest text-muted-foreground">
                        Settings
                    </p>
                </div>
                <nav className="flex flex-col overflow-y-auto p-2">
                    {CATEGORIES.map((cat) => {
                        const items = NAV_ITEMS.filter((n) => n.category === cat);
                        return (
                            <div key={cat} className="mb-1">
                                <p className="px-3 pb-1 pt-2 font-mono text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/50">
                                    {cat}
                                </p>
                                {items.map(({ key, label, icon: Icon }) => {
                                    const available = availableKeys.size === 0 || availableKeys.has(key);
                                    return (
                                        <Button
                                            key={key}
                                            onClick={() => setActive(key)}
                                            disabled={!available}
                                            variant={active === key ? "secondary" : "ghost"}
                                            size="sm"
                                            className={cn(
                                                "flex w-full items-center justify-start gap-2.5 text-left font-medium",
                                                active === key
                                                    ? "text-foreground"
                                                    : "text-muted-foreground hover:text-foreground",
                                                !available && "cursor-not-allowed opacity-40"
                                            )}
                                        >
                                            <Icon
                                                size={13}
                                                className={active === key ? "text-primary" : "text-muted-foreground"}
                                            />
                                            <span className="truncate text-[12px]">{label}</span>
                                            {active === key && (
                                                <ChevronRight size={11} className="ml-auto shrink-0 text-muted-foreground" />
                                            )}
                                        </Button>
                                    );
                                })}
                            </div>
                        );
                    })}
                </nav>
            </aside>

            {/* Content */}
            <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
                {PANEL_MAP[active]?.() ?? (
                    <div className="flex flex-1 items-center justify-center">
                        <p className="font-mono text-[11px] text-muted-foreground">
                            Select a settings category
                        </p>
                    </div>
                )}
            </main>
        </div>
    );
};
