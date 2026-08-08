import { Input } from "@/components/ui/input";
import { NamespacePanel } from "../generic-schema-panel";
import {
  Field,
  MaskedInput,
  SettingSelect,
  Toggle,
} from "../settings-primitives";
import { parseNumberInput, toStringValue } from "../settings-value-utils";

const FILE_PROC_PROVIDERS = [
  { value: "kreuzberg", label: "Kreuzberg (recommended local)" },
  { value: "auto", label: "Auto fallback" },
  { value: "unstructured", label: "Unstructured" },
  { value: "mistral", label: "Mistral OCR" },
  { value: "local", label: "Local text only" },
];

export function FileProcessingPanel() {
  return (
    <NamespacePanel namespace="file_processing" title="File Processing">
      {({ val, set }) => (
        <>
          <Field label="Default Provider">
            <SettingSelect
              value={(val("provider") as string) ?? "kreuzberg"}
              options={FILE_PROC_PROVIDERS}
              onChange={(v) => set("provider", v)}
            />
          </Field>
          <Field label="Upload Directory">
            <Input
              value={(val("upload_dir") as string) ?? ""}
              onChange={(e) => set("upload_dir", e.target.value)}
              placeholder="/tmp/uploads"
              className="font-mono text-xs"
            />
          </Field>
          <div className="grid grid-cols-2 gap-4">
            <Field label="Max Files Per Prompt">
              <Input
                type="number"
                min={1}
                value={(val("max_files_per_prompt") as number) ?? ""}
                onChange={(e) =>
                  set("max_files_per_prompt", parseInt(e.target.value))
                }
                className="font-mono text-xs"
              />
            </Field>
            <Field label="Max File Size (bytes)">
              <Input
                type="number"
                min={0}
                value={(val("max_file_size") as number) ?? ""}
                onChange={(e) => set("max_file_size", parseInt(e.target.value))}
                placeholder="10485760 (10MB)"
                className="font-mono text-xs"
              />
            </Field>
            <Field label="Max Total Size (bytes)">
              <Input
                type="number"
                min={0}
                value={(val("max_total_size") as number) ?? ""}
                onChange={(e) =>
                  set("max_total_size", parseInt(e.target.value))
                }
                placeholder="52428800 (50MB)"
                className="font-mono text-xs"
              />
            </Field>
          </div>
          <Field
            label="Allowed MIME Types"
            hint="Comma-separated allow-list used before document extraction. Leave empty to use backend defaults."
          >
            <Input
              value={toStringValue(val("allowed_mime_types"))}
              onChange={(e) =>
                set(
                  "allowed_mime_types",
                  e.target.value
                    .split(",")
                    .map((item) => item.trim())
                    .filter(Boolean),
                )
              }
              placeholder="application/pdf, text/plain, text/markdown"
              className="font-mono text-xs"
            />
          </Field>
        </>
      )}
    </NamespacePanel>
  );
}

// --- Unstructured -----------------------------------------------------------

export function UnstructuredPanel() {
  return (
    <NamespacePanel namespace="unstructured" title="Unstructured API">
      {({ val, set }) => (
        <>
          <Field label="API URL">
            <Input
              value={(val("api_url") as string) ?? ""}
              onChange={(e) => set("api_url", e.target.value)}
              placeholder="https://api.unstructuredapp.io"
              className="font-mono text-xs"
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

export function MistralOcrPanel() {
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
              className="font-mono text-xs"
            />
          </Field>
        </>
      )}
    </NamespacePanel>
  );
}

// --- Kreuzberg --------------------------------------------------------------

export function KreuzbergPanel() {
  return (
    <NamespacePanel namespace="kreuzberg" title="Kreuzberg OCR">
      {({ val, set }) => (
        <>
          <div className="space-y-3">
            {[
              {
                key: "ocr_enabled",
                label: "OCR Enabled",
                hint: "Opt in to document OCR processing",
              },
              {
                key: "force_ocr",
                label: "Force OCR",
                hint: "Always OCR even if text layer exists",
              },
              {
                key: "extract_tables",
                label: "Extract Tables",
                hint: "Preserve tables when supported by the extractor",
              },
              {
                key: "extract_metadata",
                label: "Extract Metadata",
                hint: "Capture title, language, page count, and related metadata",
              },
            ].map(({ key, label, hint }) => (
              <div
                key={key}
                className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3"
              >
                <div>
                  <p className="font-mono text-xs font-medium text-foreground">
                    {label}
                  </p>
                  <p className="font-mono text-xs text-muted-foreground">
                    {hint}
                  </p>
                </div>
                <Toggle
                  value={(val(key) as boolean) ?? false}
                  onChange={(v) => set(key, v)}
                />
              </div>
            ))}
          </div>
          <Field label="OCR Backend">
            <SettingSelect
              value={(val("ocr_backend") as string) ?? "tesseract"}
              options={[
                { value: "tesseract", label: "Tesseract" },
                { value: "paddleocr", label: "PaddleOCR" },
                { value: "easyocr", label: "EasyOCR" },
              ]}
              onChange={(v) => set("ocr_backend", v)}
            />
          </Field>
          <div className="grid gap-4 md:grid-cols-2">
            <Field label="OCR Language">
              <Input
                value={(val("ocr_language") as string) ?? "eng"}
                onChange={(e) => set("ocr_language", e.target.value)}
                placeholder="eng"
                className="font-mono text-xs"
              />
            </Field>
            <Field label="PDF DPI">
              <Input
                type="number"
                min={72}
                value={(val("pdf_dpi") as number) ?? 300}
                onChange={(e) =>
                  set("pdf_dpi", parseNumberInput(e.target.value))
                }
                className="font-mono text-xs"
              />
            </Field>
          </div>
          <Field label="Output Format">
            <SettingSelect
              value={(val("output_format") as string) ?? "markdown"}
              options={[
                { value: "markdown", label: "Markdown" },
                { value: "text", label: "Plain Text" },
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
