import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { motion } from "framer-motion";
import toast from "react-hot-toast";
import { Terminal, Database, Key, Trash2, Copy, Check } from "lucide-react";
import { SpotlightCard } from "../components/ui/SpotlightCard";
import { Button } from "../components/ui/Button";

interface LogEntry {
  text: string;
  time: string;
  type?: string;
}

export function ExtractorView() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let unlisten: () => void;
    const setupListener = async () => {
      unlisten = await listen<string>("backend-log", (event) => {
        setLogs((prev) => [...prev, { text: event.payload, time: new Date().toLocaleTimeString() }]);
      });
    };
    setupListener();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const testMount = async () => {
    setLoading(true);
    try {
      setLogs((prev) => [...prev, { text: "Starting pak mounting engine...", time: new Date().toLocaleTimeString(), type: "info" }]);
      const res = await invoke<string>("test_pak_mount");
      setLogs((prev) => [...prev, { text: res, time: new Date().toLocaleTimeString(), type: "success" }]);
      toast.success("Pak mounting completed!");
    } catch (e) {
      setLogs((prev) => [...prev, { text: String(e), time: new Date().toLocaleTimeString(), type: "error" }]);
      toast.error(String(e));
    }
    setLoading(false);
  };

  const fetchKeys = async () => {
    setLoading(true);
    try {
      setLogs((prev) => [...prev, { text: "Fetching latest AES keys from GitHub...", time: new Date().toLocaleTimeString(), type: "info" }]);
      const res = await invoke<string>("fetch_latest_keys");
      setLogs((prev) => [...prev, { text: res, time: new Date().toLocaleTimeString(), type: "success" }]);
      toast.success("Keys updated successfully");
    } catch (e) {
      setLogs((prev) => [...prev, { text: String(e), time: new Date().toLocaleTimeString(), type: "error" }]);
      toast.error(String(e));
    }
    setLoading(false);
  };

  const clearLogs = () => {
    setLogs([]);
  };

  const copyLogs = () => {
    const text = logs.map(l => `[${l.time}] ${l.text}`).join('\n');
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <motion.div 
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
      className="flex flex-col h-full overflow-hidden"
    >
      <header className="mb-4 shrink-0">
        <h1 className="text-3xl font-semibold tracking-tight text-gradient mb-1">Pak Extractor Engine</h1>
        <p className="text-fg-muted text-sm max-w-xl">
          Mounts and decodes Wuthering Waves pak archives with dynamic AES keys.
        </p>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 flex-1 min-h-0 pb-4">
        {/* Actions Bento - Spans 1 column */}
        <SpotlightCard className="col-span-1 p-5 flex flex-col h-fit">
          <div>
            <div className="w-10 h-10 rounded-xl bg-accent/10 border border-accent/20 flex items-center justify-center mb-3">
              <Key className="w-5 h-5 text-accent-bright" />
            </div>
            <h3 className="text-lg font-semibold tracking-tight mb-1.5 text-white">Pak Mounting</h3>
            <p className="text-xs text-fg-muted leading-relaxed mb-4">
              Fetches dynamic AES keys from GitHub and maps V11 GUIDs to unlock game cutscenes automatically.
            </p>
          </div>
          
          <div className="pt-4 border-t border-white/[0.06] flex flex-col gap-2">
            <Button 
              className="w-full py-2.5 text-sm font-medium" 
              onClick={testMount} 
              disabled={loading}
            >
              {loading ? "Mounting Paks..." : "Mount All Game Paks"}
            </Button>
            <Button 
              variant="secondary"
              className="w-full py-2 text-xs" 
              onClick={fetchKeys} 
              disabled={loading}
            >
              Force Refresh AES Keys
            </Button>
          </div>
        </SpotlightCard>

        <SpotlightCard className="col-span-1 md:col-span-2 flex-1 p-0 flex flex-col min-h-0 overflow-hidden border-white/10 relative" innerClassName="flex flex-col h-full min-h-0">
          {/* Header Bar */}
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/[0.06] bg-black/40 shrink-0">
            <div className="flex items-center gap-2">
              <Terminal className="w-4 h-4 text-accent" />
              <span className="text-xs font-mono tracking-wider text-fg-muted uppercase font-semibold">Process Log</span>
            </div>
            <div className="flex items-center gap-2">
              {logs.length > 0 && (
                <>
                  <button
                    onClick={copyLogs}
                    className="flex items-center gap-1.5 text-[11px] px-2 py-1 rounded bg-white/5 hover:bg-white/10 text-fg-muted hover:text-white transition-colors"
                    title="Copy logs to clipboard"
                  >
                    {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                    <span>{copied ? "Copied" : "Copy"}</span>
                  </button>
                  <button
                    onClick={clearLogs}
                    className="flex items-center gap-1.5 text-[11px] px-2 py-1 rounded bg-white/5 hover:bg-red-500/20 text-fg-muted hover:text-red-400 transition-colors"
                    title="Clear log window"
                  >
                    <Trash2 className="w-3 h-3" />
                    <span>Clear</span>
                  </button>
                </>
              )}
            </div>
          </div>
          
          {/* Log Area with Scrollbar */}
          <div className="flex-1 p-4 font-mono text-xs overflow-y-auto custom-scrollbar bg-[#08080a] leading-relaxed min-h-0 select-text cursor-text">
            {logs.length === 0 ? (
              <div className="h-full flex flex-col items-center justify-center text-fg-subtle/40 gap-2">
                <Database className="w-8 h-8 opacity-40" />
                <p className="text-xs font-sans">Ready. Click "Mount All Game Paks" to begin.</p>
              </div>
            ) : (
              <div className="flex flex-col gap-1.5">
                {logs.map((log, i) => (
                  <div key={i} className={`flex items-start gap-2.5 break-all ${
                    log.type === 'error' ? 'text-red-400 font-semibold' : 
                    log.type === 'success' ? 'text-emerald-400 font-semibold' : 
                    log.type === 'info' ? 'text-accent-bright' : 'text-fg-muted'
                  }`}>
                    <span className="opacity-40 shrink-0 text-[10px] select-none">[{log.time}]</span>
                    <span>{log.text}</span>
                  </div>
                ))}
                <div ref={logEndRef} />
              </div>
            )}
          </div>
        </SpotlightCard>
      </div>
    </motion.div>
  );
}
