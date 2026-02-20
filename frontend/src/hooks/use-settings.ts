import { useCallback, useEffect, useRef, useState } from "react";
import type { SettingWithMeta } from "@/types";

export interface UseSettingsReturn {
    /** Map of leaf key → current value (e.g. "resilience.rate_limit_enabled" → true) */
    values: Record<string, unknown>;
    /** Full setting objects keyed by leaf key */
    settings: Record<string, SettingWithMeta>;
    loading: boolean;
    saving: boolean;
    error: string | null;
    /** Update a value in local state immediately */
    setSetting: (key: string, value: unknown) => void;
    /** Persist all pending dirty values to the API */
    saveAll: () => Promise<void>;
    /** Reload from the API */
    reload: () => Promise<void>;
}

/** Convert a namespace key (e.g. "context_management") to its URL slug (e.g. "context-management") */
function namespaceToSlug(ns: string): string {
    const overrides: Record<string, string> = {
        provider: "providers",
        file_processing: "file-processing",
        knowledge_bases: "knowledge-bases",
        intent_classifier: "intent-classifier",
        context_management: "context-management",
        agent_config: "agent-config",
        skill_config: "skill-config",
        mistral_ocr: "mistral-ocr",
    };
    return overrides[ns] ?? ns.replace(/_/g, "-");
}

const BASE = "/api/uar/settings";

export function useSettings(namespace: string): UseSettingsReturn {
    const [settings, setSettings] = useState<Record<string, SettingWithMeta>>({});
    const [values, setValues] = useState<Record<string, unknown>>({});
    const [dirty, setDirty] = useState<Record<string, unknown>>({});
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const savingRef = useRef(false);

    const load = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const slug = namespaceToSlug(namespace);
            const res = await fetch(`${BASE}/${slug}`);
            if (!res.ok) throw new Error(`${res.status}`);
            const data = (await res.json()) as SettingWithMeta[];
            const byKey: Record<string, SettingWithMeta> = {};
            const vals: Record<string, unknown> = {};
            for (const s of data) {
                byKey[s.key] = s;
                vals[s.key] = s.data;
            }
            setSettings(byKey);
            setValues(vals);
            setDirty({});
        } catch (e) {
            setError((e as Error).message);
        } finally {
            setLoading(false);
        }
    }, [namespace]);

    useEffect(() => {
        void load();
    }, [load]);

    const setSetting = useCallback((key: string, value: unknown) => {
        setValues((prev) => ({ ...prev, [key]: value }));
        setDirty((prev) => ({ ...prev, [key]: value }));
    }, []);

    const saveAll = useCallback(async () => {
        if (savingRef.current || Object.keys(dirty).length === 0) return;
        savingRef.current = true;
        setSaving(true);
        setError(null);
        try {
            await Promise.all(
                Object.entries(dirty).map(([key, value]) =>
                    fetch(`${BASE}/${encodeURIComponent(key)}`, {
                        method: "PUT",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ value }),
                    }).then((r) => {
                        if (!r.ok) throw new Error(`Save ${key} failed: ${r.status}`);
                    })
                )
            );
            setDirty({});
        } catch (e) {
            setError((e as Error).message);
        } finally {
            savingRef.current = false;
            setSaving(false);
        }
    }, [dirty]);

    return { values, settings, loading, saving, error, setSetting, saveAll, reload: load };
}
