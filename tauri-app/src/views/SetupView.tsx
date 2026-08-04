import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { motion } from "framer-motion";
import toast from "react-hot-toast";
import { Download, FolderOpen, CheckCircle2, AlertCircle, Loader2, Settings2 } from "lucide-react";
import { SpotlightCard } from "../components/ui/SpotlightCard";
import { Button } from "../components/ui/Button";

interface AppConfig {
  ffmpeg_path: string;
  vgmstream_path: string;
  game_dir: string;
  wwise_path: string;
  char_selection: string;
  locale_selection: string;
  subtitle_lang: string;
  subtitle_mode: string;
}

interface ProgressPayload {
  task: string;
  progress: number;
}

interface DependencyStatus {
  ffmpeg: boolean;
  wwiser: boolean;
  vgmstream: boolean;
  json_data: boolean;
}

export function SetupView() {
  const [config, setConfig] = useState<AppConfig>({
    ffmpeg_path: "",
    vgmstream_path: "",
    game_dir: "",
    wwise_path: "",
    char_selection: "Girl",
    locale_selection: "ja",
    subtitle_lang: "en",
    subtitle_mode: "Soft-sub",
  });
  
  const [downloading, setDownloading] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [progress, setProgress] = useState(0);
  const [currentTask, setCurrentTask] = useState("");
  const [deps, setDeps] = useState<DependencyStatus | null>(null);

  const checkDeps = async () => {
    try {
      const status = await invoke<DependencyStatus>("check_dependencies");
      setDeps(status);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
    checkDeps();

    const unlistenLog = listen<string>("backend-log", (event) => {
      const text = typeof event.payload === "string" ? event.payload : JSON.stringify(event.payload);
      setLogs((prev) => [...prev.slice(-4), text]);
    });
    const unlistenProgress = listen<ProgressPayload>("downloader-progress", (event) => {
      setCurrentTask(event.payload.task);
      setProgress(event.payload.progress * 100);
    });

    return () => {
      unlistenLog.then((f) => f());
      unlistenProgress.then((f) => f());
    };
  }, []);

  const updateConfig = async (newConfig: AppConfig) => {
    setConfig(newConfig);
    try {
      await invoke("update_config", { newConfig });
    } catch (e) {
      console.error(e);
    }
  };

  const selectDir = async (key: keyof AppConfig) => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      updateConfig({ ...config, [key]: selected });
    }
  };

  const downloadAll = async () => {
    setDownloading(true);
    setLogs([]);
    try {
      if (!deps?.ffmpeg || !deps?.wwiser || !deps?.vgmstream) {
        await invoke("download_tools");
      }
      if (!deps?.json_data) {
        await invoke("download_data", { textmapLang: config.subtitle_lang || "en" });
      }
      toast.success("All dependencies ready!");
    } catch (e) {
      toast.error(String(e));
      console.error(e);
    } finally {
      setDownloading(false);
      setProgress(0);
      setCurrentTask("");
      checkDeps();
    }
  };
  const isAllReady = deps?.ffmpeg && deps?.wwiser && deps?.vgmstream && deps?.json_data;

  return (
    <motion.div 
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
      className="h-full flex flex-col p-8 lg:p-12 overflow-y-auto custom-scrollbar"
    >
      <div className="max-w-4xl w-full mx-auto space-y-8">
        
        <header>
          <div className="flex items-center gap-3 mb-2">
            <div className="w-10 h-10 rounded-xl bg-accent/10 border border-accent/20 flex items-center justify-center shadow-inner-glow">
              <Settings2 className="w-5 h-5 text-accent-bright" />
            </div>
            <div>
              <h1 className="text-2xl font-bold tracking-tight text-fg">Environment Setup</h1>
              <p className="text-sm text-fg-muted mt-1">Configure paths and download required dependencies for processing cutscenes.</p>
            </div>
          </div>
        </header>

        <SpotlightCard className="p-10 flex flex-col gap-12 border-white/5">
          {/* Paths Section */}
          <div className="space-y-6 pt-2">
            <h2 className="text-sm font-semibold tracking-wide text-fg uppercase flex items-center gap-2">
              <span className="w-5 h-5 rounded-md bg-accent/20 text-accent-bright flex items-center justify-center text-[10px]">1</span>
              Configuration Paths
            </h2>
            
            <div className="space-y-4">
              <div className="space-y-2">
                <label className="block text-xs font-medium text-fg-muted uppercase tracking-wider">Game Directory</label>
                <p className="text-[10px] text-fg-subtle -mt-1">Root folder of the game, e.g. <code className="bg-white/5 px-1 rounded">D:\Game\Wuthering Waves Game</code></p>
                <div className="flex gap-2">
                  <input 
                    type="text" 
                    value={config.game_dir} 
                    className="flex-1 bg-white/5 border border-white/10 rounded-lg px-4 py-2 text-sm focus:outline-none focus:border-accent/50 transition-all text-fg" 
                    placeholder="Select game root directory..." 
                    readOnly 
                  />
                  <Button variant="secondary" onClick={() => selectDir("game_dir")} className="px-4">
                    <FolderOpen className="w-4 h-4 mr-2" />
                    Browse
                  </Button>
                </div>
              </div>
              

            </div>
          </div>

          <div className="w-full h-px bg-white/5" />

          {/* Preferences Section */}
          <div className="space-y-6 pt-2">
            <h2 className="text-sm font-semibold tracking-wide text-fg uppercase flex items-center gap-2">
              <span className="w-5 h-5 rounded-md bg-accent/20 text-accent-bright flex items-center justify-center text-[10px]">2</span>
              Preferences
            </h2>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="block text-xs font-medium text-fg-muted uppercase tracking-wider">Rover Gender</label>
                <div className="flex bg-white/5 p-1 rounded-lg border border-white/5">
                  <button 
                    onClick={() => updateConfig({...config, char_selection: "Girl"})}
                    className={`flex-1 py-1.5 text-xs rounded-md font-medium transition-all ${
                      config.char_selection === "Girl" ? 'bg-accent text-white shadow-sm' : 'text-fg-muted hover:text-fg'
                    }`}
                  >Girl</button>
                  <button 
                    onClick={() => updateConfig({...config, char_selection: "Boy"})}
                    className={`flex-1 py-1.5 text-xs rounded-md font-medium transition-all ${
                      config.char_selection === "Boy" ? 'bg-accent text-white shadow-sm' : 'text-fg-muted hover:text-fg'
                    }`}
                  >Boy</button>
                </div>
              </div>

              <div className="space-y-2">
                <label className="block text-xs font-medium text-fg-muted uppercase tracking-wider">Voice Locale</label>
                <select
                  value={config.locale_selection}
                  onChange={(e) => updateConfig({...config, locale_selection: e.target.value})}
                  className="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-1.5 text-xs text-fg outline-none focus:border-accent/50 appearance-none"
                >
                  <option value="ja" className="bg-bg-deep text-fg">Japanese (ja)</option>
                  <option value="en" className="bg-bg-deep text-fg">English (en)</option>
                  <option value="ko" className="bg-bg-deep text-fg">Korean (ko)</option>
                  <option value="zh" className="bg-bg-deep text-fg">Chinese (zh)</option>
                </select>
              </div>

              <div className="space-y-2">
                <label className="block text-xs font-medium text-fg-muted uppercase tracking-wider">Subtitle Language</label>
                <select
                  value={config.subtitle_lang}
                  onChange={(e) => updateConfig({...config, subtitle_lang: e.target.value})}
                  className="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-1.5 text-xs text-fg outline-none focus:border-accent/50 appearance-none"
                >
                  <option value="en" className="bg-bg-deep text-fg">English</option>
                  <option value="ja" className="bg-bg-deep text-fg">Japanese</option>
                  <option value="th" className="bg-bg-deep text-fg">Thai</option>
                  <option value="ko" className="bg-bg-deep text-fg">Korean</option>
                  <option value="zh" className="bg-bg-deep text-fg">Chinese</option>
                </select>
              </div>

              <div className="space-y-2">
                <label className="block text-xs font-medium text-fg-muted uppercase tracking-wider">Subtitle Mode</label>
                <select
                  value={config.subtitle_mode}
                  onChange={(e) => updateConfig({...config, subtitle_mode: e.target.value})}
                  className="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-1.5 text-xs text-fg outline-none focus:border-accent/50 appearance-none"
                >
                  <option value="None" className="bg-bg-deep text-fg">None</option>
                  <option value="Soft-sub" className="bg-bg-deep text-fg">Soft-sub (Muxed track)</option>
                  <option value="Hard-sub" className="bg-bg-deep text-fg">Hard-sub (Burned-in)</option>
                </select>
              </div>
            </div>
          </div>

          <div className="w-full h-px bg-white/5" />

          {/* Dependencies Section */}
          <div className="space-y-6 pt-2">
            <h2 className="text-sm font-semibold tracking-wide text-fg uppercase flex items-center gap-2">
              <span className="w-5 h-5 rounded-md bg-accent/20 text-accent-bright flex items-center justify-center text-[10px]">3</span>
              Required Dependencies
            </h2>

            <div className="grid grid-cols-2 gap-4">
              <DependencyItem name="FFmpeg" isReady={deps?.ffmpeg} />
              <DependencyItem name="wwiser (Python)" isReady={deps?.wwiser} />
              <DependencyItem name="vgmstream" isReady={deps?.vgmstream} />
              <DependencyItem name="Cutscene JSON Data" isReady={deps?.json_data} />
            </div>

            {downloading && (
              <div className="mt-4 bg-white/5 border border-white/10 rounded-xl p-6">
                <div className="flex items-center justify-between mb-4">
                  <h4 className="text-sm font-medium text-fg">{currentTask || "Preparing download..."}</h4>
                  <span className="text-xs font-mono text-accent-bright">{Math.round(progress)}%</span>
                </div>
                <div className="h-1.5 bg-black/40 rounded-full overflow-hidden">
                  <motion.div 
                    className="h-full bg-accent-bright"
                    initial={{ width: 0 }}
                    animate={{ width: `${progress}%` }}
                    transition={{ ease: "easeOut", duration: 0.2 }}
                  />
                </div>
                
                <div className="mt-4 pt-4 border-t border-white/5 space-y-1">
                  {logs.map((log, i) => (
                    <div key={i} className="text-[10px] font-mono text-fg-subtle truncate">
                      {log}
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="flex items-center justify-between pt-4">
              <div className="text-sm">
                {isAllReady ? (
                  <span className="text-emerald-400 flex items-center gap-2">
                    <CheckCircle2 className="w-4 h-4" /> All dependencies are ready
                  </span>
                ) : (
                  <span className="text-amber-400 flex items-center gap-2">
                    <AlertCircle className="w-4 h-4" /> Missing dependencies
                  </span>
                )}
              </div>
              
              {!isAllReady && (
                <Button 
                  onClick={downloadAll} 
                  disabled={downloading}
                >
                  {downloading ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Downloading...
                    </>
                  ) : (
                    <>
                      <Download className="w-4 h-4 mr-2" />
                      Download Missing
                    </>
                  )}
                </Button>
              )}
            </div>
          </div>

        </SpotlightCard>
      </div>
    </motion.div>
  );
}

function DependencyItem({ name, isReady }: { name: string, isReady?: boolean }) {
  return (
    <div className="bg-white/[0.02] border border-white/[0.06] rounded-xl p-4 flex items-center justify-between">
      <span className="text-sm font-medium text-fg">{name}</span>
      {isReady === undefined ? (
        <Loader2 className="w-4 h-4 text-fg-subtle animate-spin" />
      ) : isReady ? (
        <div className="bg-emerald-500/10 text-emerald-400 px-2 py-1 rounded text-[10px] font-bold uppercase tracking-wider border border-emerald-500/20">
          Installed
        </div>
      ) : (
        <div className="bg-amber-500/10 text-amber-400 px-2 py-1 rounded text-[10px] font-bold uppercase tracking-wider border border-amber-500/20">
          Missing
        </div>
      )}
    </div>
  );
}
