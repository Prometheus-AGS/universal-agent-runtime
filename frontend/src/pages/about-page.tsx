import { Bot, Code2, Database, Globe, Layers, Shield, Zap } from "lucide-react";
import { KnowMeLogo, KnowMeWordmark } from "@/components/KnowMeLogo";
import { useHealthz } from "@/hooks/use-healthz";

export function AboutPage() {
  const { health } = useHealthz();

  const features = [
    { icon: Zap, title: "Streaming Agent Runtime", desc: "Real-time SSE streaming with AG-UI dual-mode event protocol and full tool-call visibility." },
    { icon: Layers, title: "MCP Tool Integration", desc: "Dynamic tool discovery from mcp.json supporting both stdio and HTTP-based MCP servers." },
    { icon: Database, title: "Multi-Provider Support", desc: "OpenAI, Anthropic, Google, Ollama, and any OpenAI-compatible endpoint." },
    { icon: Zap, title: "Skill Engine", desc: "Keyword, embedding, hybrid, and LLM-based skill selection for contextual agent capabilities." },
    { icon: Code2, title: "WASM Extensibility", desc: "Compiler sessions for packaging and deploying WASM skill modules at runtime." },
    { icon: Shield, title: "API Key Auth", desc: "Fine-grained API key management for controlling access to the agent runtime." },
    { icon: Globe, title: "Tauri Compatible", desc: "Serves identically in browser and Tauri desktop/mobile contexts via static SPA hosting." },
    { icon: Bot, title: "Agent Federation", desc: "Compose runtime and federated agents with configurable skill sets and fallback chains." },
  ];

  return (
    <div className="flex flex-1 flex-col overflow-y-auto">
      <div className="mx-auto w-full max-w-4xl px-6 py-12">
        {/* Hero */}
        <div className="mb-12 text-center">
          <div className="mx-auto mb-6 flex size-20 items-center justify-center rounded-3xl bg-card">
            <KnowMeLogo size={44} className="text-foreground" />
          </div>
          <h1 className="font-display text-4xl font-bold tracking-tight text-foreground"><KnowMeWordmark /></h1>
          <p className="mt-3 font-mono text-sm text-primary">{"Agentic streaming AI infrastructure · powered by the Universal Agent Runtime"}</p>

          {health && (
            <div className="mt-4 inline-flex items-center gap-2 rounded-full border border-success/30 bg-success/10 px-4 py-1.5">
              <div className="size-2 rounded-full bg-success animate-pulse" />
              <span className="font-mono text-[11px] text-success">
                {health.status ?? "online"}{health.version ? ` · v${health.version}` : ""}
              </span>
            </div>
          )}
        </div>

        {/* Description */}
        <div className="mb-12 rounded-xl border border-border bg-card p-6">
          <p className="font-body text-base leading-relaxed text-foreground">
            The Universal Agent Runtime is an agentic streaming LLM application built on{" "}
            <span className="text-primary">Rust (Axum)</span> with a{" "}
            <span className="text-primary">React + Vite</span> frontend. It provides tool-first,
            streaming-native agent infrastructure with MCP tool integration, multi-provider LLM
            routing, and a full administration interface.
          </p>
          <p className="mt-4 font-body text-base leading-relaxed text-muted-foreground">
            Designed for web, desktop (Tauri), and mobile deployment from a single codebase.
            All agent interactions stream via Server-Sent Events using the AG-UI protocol,
            providing real-time visibility into reasoning, tool calls, context management, and skill activation.
          </p>
        </div>

        {/* Features grid */}
        <h2 className="mb-4 font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">Capabilities</h2>
        <div className="mb-12 grid gap-4 sm:grid-cols-2 lg:grid-cols-2">
          {features.map(({ icon: Icon, title, desc }) => (
            <div key={title} className="rounded-xl border border-border bg-card p-5">
              <div className="mb-3 flex size-9 items-center justify-center rounded-lg bg-primary/15">
                <Icon size={16} className="text-primary" />
              </div>
              <h3 className="font-display text-[14px] font-semibold text-foreground">{title}</h3>
              <p className="mt-1.5 font-body text-sm text-muted-foreground leading-relaxed">{desc}</p>
            </div>
          ))}
        </div>

        {/* Tech stack */}
        <h2 className="mb-4 font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">Stack</h2>
        <div className="mb-12 overflow-hidden rounded-xl border border-border">
          {[
            ["Backend", "Rust · Axum · Leptos SSR · rmcp · tokio · mimalloc"],
            ["Frontend", "React 19 · Vite 7 · TypeScript · Tailwind CSS · Zustand"],
            ["Streaming", "Server-Sent Events · AG-UI protocol · dual stream mode"],
            ["AI", "OpenAI Chat Completions API · MCP (Model Context Protocol)"],
            ["Persistence", "SQLite · Zustand localStorage · thread registry"],
          ].map(([label, value], i, arr) => (
            <div key={label} className={`flex items-start gap-4 px-5 py-3.5 ${i < arr.length - 1 ? "border-b border-border" : ""}`}>
              <span className="w-24 shrink-0 font-mono text-[10px] font-medium uppercase tracking-widest text-muted-foreground">{label}</span>
              <span className="font-body text-sm text-foreground">{value}</span>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="text-center">
          <p className="font-mono text-[10px] text-muted-foreground/50">Universal Agent Runtime · Built with Rust + React</p>
        </div>
      </div>
    </div>
  );
}
