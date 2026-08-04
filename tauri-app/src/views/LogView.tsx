import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { motion } from "framer-motion";
import { ScrollText, Trash2, Copy, Check, Filter } from "lucide-react";
import { SpotlightCard } from "../components/ui/SpotlightCard";

interface LogEntry {
  text: string;
  time: string;
  type: "info" | "warn" | "error" | "success";
}

function classifyLog(text: string): LogEntry["type"] {
  const lower = text.toLowerCase();
  if (lower.startsWith("success") || lower.startsWith("finished") || lower.includes("successfully") || lower.includes("completed")) return "success";
  if (lower.includes("error:") || lower.startsWith("error") || lower.includes("failed")) return "error";
  if (lower.includes("warning") || lower.startsWith("⚠")) return "warn";
  return "info";
}

type FilterType = "all" | "info" | "warn" | "error" | "success";

export function LogView() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [copied, setCopied] = useState(false);
  const [filter, setFilter] = useState<FilterType>("all");
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let unlisten: () => void;
    const setup = async () => {
      unlisten = await listen<string>("backend-log", (event) => {
        const text = typeof event.payload === "string" ? event.payload : JSON.stringify(event.payload);
        setLogs((prev) => [
          ...prev,
          {
            text,
            time: new Date().toLocaleTimeString(),
            type: classifyLog(text),
          },
        ]);
      });
    };
    setup();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const clearLogs = () => setLogs([]);

  const copyLogs = () => {
    const text = logs.map((l) => `[${l.time}] ${l.text}`).join("\n");
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const filteredLogs = filter === "all" ? logs : logs.filter((l) => l.type === filter);

  const FILTERS: { key: FilterType; label: string }[] = [
    { key: "all", label: "All" },
    { key: "error", label: "Errors" },
    { key: "warn", label: "Warnings" },
    { key: "success", label: "Success" },
    { key: "info", label: "Info" },
  ];

  const logTypeColors: Record<LogEntry["type"], string> = {
    error: "text-red-400",
    warn: "text-amber-400",
    success: "text-emerald-400",
    info: "text-fg-muted",
  };

  const errorCount = logs.filter((l) => l.type === "error").length;
  const warnCount = logs.filter((l) => l.type === "warn").length;

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
      className="flex flex-col h-full overflow-hidden"
    >
      <header className="mb-4 flex items-center justify-between shrink-0">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight text-gradient mb-1">Activity Log</h1>
          <p className="text-fg-muted text-xs">
            {logs.length} entries
            {errorCount > 0 && <span className="text-red-400 ml-2">• {errorCount} errors</span>}
            {warnCount > 0 && <span className="text-amber-400 ml-2">• {warnCount} warnings</span>}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {logs.length > 0 && (
            <>
              <button
                onClick={copyLogs}
                className="flex items-center gap-1.5 text-[11px] px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-fg-muted hover:text-white transition-colors border border-white/[0.06]"
                title="Copy all logs"
              >
                {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                <span>{copied ? "Copied" : "Copy"}</span>
              </button>
              <button
                onClick={clearLogs}
                className="flex items-center gap-1.5 text-[11px] px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-red-500/20 text-fg-muted hover:text-red-400 transition-colors border border-white/[0.06]"
                title="Clear all logs"
              >
                <Trash2 className="w-3 h-3" />
                <span>Clear</span>
              </button>
            </>
          )}
        </div>
      </header>

      {/* Filter Bar */}
      <div className="flex items-center gap-1 mb-3 shrink-0">
        <Filter className="w-3.5 h-3.5 text-fg-subtle mr-1" />
        {FILTERS.map((f) => (
          <button
            key={f.key}
            onClick={() => setFilter(f.key)}
            className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-all ${
              filter === f.key
                ? "bg-accent/20 text-accent-bright border border-accent/30"
                : "bg-white/[0.03] text-fg-muted hover:bg-white/[0.06] border border-transparent"
            }`}
          >
            {f.label}
            {f.key !== "all" && (
              <span className="ml-1 opacity-60">
                ({logs.filter((l) => l.type === f.key).length})
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Log Content */}
      <SpotlightCard className="flex-1 p-0 flex flex-col min-h-0 overflow-hidden border-white/10" innerClassName="flex flex-col h-full min-h-0">
        <div className="flex items-center justify-between px-4 py-2 border-b border-white/[0.06] bg-black/40 shrink-0">
          <div className="flex items-center gap-2">
            <ScrollText className="w-4 h-4 text-accent" />
            <span className="text-xs font-mono tracking-wider text-fg-muted uppercase font-semibold">
              Full Log Output
            </span>
          </div>
          <span className="text-[10px] font-mono text-fg-subtle">{filteredLogs.length} entries</span>
        </div>

        <div className="flex-1 p-4 font-mono text-xs overflow-y-auto custom-scrollbar bg-[#08080a] leading-relaxed min-h-0 select-text cursor-text">
          {filteredLogs.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-fg-subtle/40 gap-2">
              <ScrollText className="w-8 h-8 opacity-40" />
              <p className="text-xs font-sans">
                {logs.length === 0
                  ? "No logs recorded yet. Perform an action to see logs here."
                  : "No logs match the selected filter."}
              </p>
            </div>
          ) : (
            <div className="flex flex-col gap-1">
              {filteredLogs.map((log, i) => (
                <div
                  key={i}
                  className={`flex items-start gap-2.5 break-all ${logTypeColors[log.type]}`}
                >
                  <span className="opacity-40 shrink-0 text-[10px] select-none">[{log.time}]</span>
                  <span>{log.text}</span>
                </div>
              ))}
              <div ref={logEndRef} />
            </div>
          )}
        </div>
      </SpotlightCard>
    </motion.div>
  );
}
