import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import toast from "react-hot-toast";
import { RefreshCw, FileVideo, Film, Settings2, Play, CheckCircle2, AlertCircle, Loader2, Search, FolderOpen } from "lucide-react";
import { SpotlightCard } from "../components/ui/SpotlightCard";
import { Button } from "../components/ui/Button";

type ProcessStatus = "idle" | "extracting" | "generating" | "rendering" | "done" | "error";

export function GalleryView() {
  const [files, setFiles] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [currentPath, setCurrentPath] = useState("");
  
  // Selection
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [gender, setGender] = useState<"Girl" | "Boy">("Girl");
  const [locale, setLocale] = useState("en");
  const [subtitleMode, setSubtitleMode] = useState<"None" | "Soft-sub" | "Hard-sub">("None");
  
  // Processing State
  const [status, setStatus] = useState<ProcessStatus>("idle");
  const [log, setLog] = useState("");
  const [processLogs, setProcessLogs] = useState<string[]>([]);
  const [audioEvents, setAudioEvents] = useState<string[]>([]);

  useEffect(() => {
    if (selectedFile) {
      invoke<string[]>("get_video_audio_event", { videoName: selectedFile })
        .then(setAudioEvents)
        .catch(console.error);
    } else {
      setAudioEvents([]);
    }
  }, [selectedFile]);
  
  const fetchFiles = async () => {
    setLoading(true);
    try {
      const result = await invoke<string[]>("get_pak_files");
      setFiles(result);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchFiles();
    
    // Listen to backend logs for detailed process view
    let unlisten: any;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<string>("backend-log", (event) => {
        setProcessLogs(prev => [...prev.slice(-30), event.payload]);
      }).then(u => unlisten = u);
    });
    
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const filteredFiles = files.map(f => f.replace(/\\/g, '/')).filter(f => 
    f.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const getDirectoryContents = () => {
    if (searchQuery) {
      return { folders: [], displayFiles: filteredFiles };
    }

    const folders = new Set<string>();
    const currentFiles: string[] = [];

    filteredFiles.forEach(f => {
      const prefix = currentPath ? currentPath + '/' : '';
      if (f.startsWith(prefix) || currentPath === "") {
        const remaining = f.substring(prefix.length);
        const slashIndex = remaining.indexOf('/');
        if (slashIndex !== -1) {
          folders.add(remaining.substring(0, slashIndex));
        } else {
          currentFiles.push(f);
        }
      }
    });

    return {
      folders: Array.from(folders).sort(),
      displayFiles: currentFiles.sort()
    };
  };

  const { folders, displayFiles } = getDirectoryContents();

  const handleProcess = async () => {
    if (!selectedFile) return;
    
    try {
      setProcessLogs([]);
      setStatus("extracting");
      setLog(`Extracting ${selectedFile.split('/').pop() || selectedFile} from Pak...`);
      const mp4Path = await invoke<string>("process_cutscene", { videoName: selectedFile });
      
      setStatus("generating");
      setLog("Extracting audio banks from Paks...");
      await invoke("extract_audio_banks");

      setLog("Mapping audio banks...");
      await invoke("run_wwiser"); // Run Wwiser fully automated
      
      setLog(`Matching audio for ${gender}, locale (${locale})...`);
      const videoInfo = await invoke("generate_single_video_info", { 
        mp4Path, 
        locale, 
        girlOrBoy: gender 
      });
      
      setStatus("rendering");
      setLog(`Rendering video with FFmpeg (${subtitleMode})...`);
      await invoke("render_video", { videoInfo, subtitleMode });
      
      setStatus("done");
      setLog(`Successfully processed ${selectedFile.split('/').pop() || selectedFile}!`);
      toast.success("Video processed successfully!");
      
    } catch (err: any) {
      console.error(err);
      setStatus("error");
      setLog(err.toString());
      toast.error(err.toString());
    }
  };

  const handleOpenOutput = async () => {
    try {
      await invoke("open_output_dir");
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <motion.div 
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
      className="flex flex-col h-full overflow-hidden"
    >
      <header className="mb-4 flex items-center justify-between shrink-0">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight text-gradient mb-1">Cutscene Gallery</h1>
          <p className="text-fg-muted text-xs">
            Select a sequence, configure audio & subtitles, and render.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {/* Search Box */}
          <div className="relative w-64">
            <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-fg-subtle" />
            <input 
              type="text"
              placeholder="Search cutscenes..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-white/5 border border-white/10 rounded-xl pl-9 pr-3 py-1.5 text-xs text-fg outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/50 transition-all placeholder:text-fg-subtle/50"
            />
          </div>

          <Button onClick={handleOpenOutput} variant="secondary" className="py-1.5 px-3 text-xs bg-white/5 hover:bg-white/10 border-white/10">
            <FolderOpen className="w-3.5 h-3.5 mr-1.5 text-accent-bright" />
            Output Folder
          </Button>

          <Button onClick={fetchFiles} disabled={loading} variant="secondary" className="py-1.5 px-3 text-xs">
            <RefreshCw className={`w-3.5 h-3.5 mr-1.5 ${loading ? 'animate-spin' : ''}`} />
            {loading ? "Loading..." : "Refresh"}
          </Button>
        </div>
      </header>

      <div className="flex flex-row gap-4 flex-1 overflow-hidden pb-4">
        
        {/* Left Side: Video Grid */}
        <div className="flex-1 flex flex-col overflow-hidden pr-2">
          
          {/* Breadcrumbs */}
          {!searchQuery && (
            <div className="flex flex-wrap items-center gap-1.5 mb-3 text-[11px] font-medium text-fg-muted shrink-0">
              <button 
                onClick={() => setCurrentPath("")} 
                className={`hover:text-fg transition-colors ${currentPath === "" ? 'text-accent-bright' : ''}`}
              >
                Root
              </button>
              {currentPath.split('/').filter(Boolean).map((part, i, arr) => {
                 const pathSoFar = arr.slice(0, i+1).join('/');
                 const isLast = i === arr.length - 1;
                 return (
                   <div key={pathSoFar} className="flex items-center gap-1.5">
                     <span className="opacity-40">/</span>
                     <button 
                       onClick={() => setCurrentPath(pathSoFar)}
                       className={`hover:text-fg transition-colors ${isLast ? 'text-accent-bright' : ''}`}
                     >
                       {part}
                     </button>
                   </div>
                 );
              })}
            </div>
          )}

          <div className="flex-1 overflow-y-auto custom-scrollbar">
            {folders.length === 0 && displayFiles.length === 0 && !loading ? (
              <SpotlightCard className="h-64 flex flex-col items-center justify-center text-center p-6 border-dashed border-white/10">
                <div className="w-12 h-12 rounded-2xl bg-white/[0.02] border border-white/5 flex items-center justify-center mb-3">
                  <Film className="w-6 h-6 text-fg-subtle opacity-50" />
                </div>
                <h3 className="text-base font-medium text-fg mb-1">No videos found</h3>
                <p className="text-xs text-fg-muted max-w-xs">
                  {searchQuery ? "No cutscenes match your search query." : "Please mount the pak files in the Mount Paks tab first."}
                </p>
              </SpotlightCard>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 pb-4">
                {/* Folders */}
                {folders.map((folder, i) => {
                  const fullPath = currentPath ? `${currentPath}/${folder}` : folder;
                  return (
                    <motion.div key={fullPath} initial={{opacity:0, scale: 0.96}} animate={{opacity:1, scale: 1}} transition={{ duration: 0.2, delay: i * 0.01 }}>
                      <SpotlightCard 
                        onClick={() => setCurrentPath(fullPath)}
                        className="p-3 flex items-center gap-3 group cursor-pointer hover:bg-white/[0.04] transition-all duration-200"
                      >
                        <div className="shrink-0 w-10 h-10 rounded-xl bg-accent/10 border border-accent/20 text-accent-bright flex items-center justify-center group-hover:bg-accent/20 transition-colors">
                          <FolderOpen className="w-4 h-4" />
                        </div>
                        <div className="flex-1 min-w-0 overflow-hidden">
                          <p className="text-xs font-medium text-fg group-hover:text-white truncate transition-colors">{folder}</p>
                          <p className="text-[10px] text-fg-subtle font-mono truncate opacity-60">Folder</p>
                        </div>
                      </SpotlightCard>
                    </motion.div>
                  )
                })}

                {/* Files */}
                {displayFiles.map((file, i) => {
                  const isSelected = selectedFile === file;
                  const fileName = file.split('/').pop() || file.split('\\').pop() || file;
                  
                  return (
                    <motion.div
                      initial={{ opacity: 0, scale: 0.96 }}
                      animate={{ opacity: 1, scale: 1 }}
                      transition={{ duration: 0.2, delay: (folders.length + i) * 0.01 }}
                      key={file}
                    >
                      <SpotlightCard 
                        onClick={() => setSelectedFile(file)}
                        className={`p-3 flex items-center gap-3 group cursor-pointer transition-all duration-200 ${
                          isSelected ? 'ring-2 ring-accent bg-accent/10 border-accent/40' : 'hover:bg-white/[0.04]'
                        }`}
                      >
                        <div className={`shrink-0 w-10 h-10 rounded-xl flex items-center justify-center transition-colors ${
                          isSelected ? 'bg-accent text-white' : 'bg-white/5 text-fg-muted group-hover:text-white'
                        }`}>
                          <FileVideo className="w-4 h-4" />
                        </div>
                        <div className="flex-1 min-w-0 overflow-hidden">
                          <p className={`text-xs font-medium truncate transition-colors ${isSelected ? 'text-white font-semibold' : 'text-fg group-hover:text-white'}`}>
                            {fileName}
                          </p>
                          <p className="text-[10px] text-fg-subtle font-mono truncate opacity-60">
                            {file}
                          </p>
                        </div>
                      </SpotlightCard>
                    </motion.div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
        
        {/* Right Side: Configuration & Processing Panel */}
        <AnimatePresence mode="wait">
          {selectedFile && (
            <motion.div 
              initial={{ opacity: 0, x: 20, width: 0 }}
              animate={{ opacity: 1, x: 0, width: "300px" }}
              exit={{ opacity: 0, x: 20, width: 0 }}
              className="shrink-0 h-full flex flex-col"
            >
              <SpotlightCard className="h-full flex flex-col p-4 border-white/10 shadow-card">
                <div className="flex items-center gap-2 mb-4 border-b border-white/10 pb-3 shrink-0">
                  <div className="w-8 h-8 rounded-lg bg-accent/10 border border-accent/20 flex items-center justify-center">
                    <Settings2 className="w-4 h-4 text-accent-bright" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h2 className="text-sm font-semibold text-fg">Process Options</h2>
                    <p className="text-[11px] text-fg-muted truncate" title={selectedFile}>
                      {selectedFile.split('/').pop() || selectedFile}
                    </p>
                    {audioEvents.length > 0 && (
                      <p className="text-[10px] text-accent-bright font-mono truncate mt-1">
                        Audio: {audioEvents.join(", ")}
                      </p>
                    )}
                  </div>
                </div>

                <div className="flex flex-col gap-3 overflow-y-auto custom-scrollbar pr-1 flex-1">
                  <div>
                    <label className="text-[10px] font-semibold text-fg-muted uppercase tracking-wider mb-1.5 block">Gender (Rover)</label>
                    <div className="flex bg-white/5 p-1 rounded-lg border border-white/5">
                      <button 
                        onClick={() => setGender("Girl")}
                        disabled={status !== "idle" && status !== "error" && status !== "done"}
                        className={`flex-1 py-1 text-xs rounded-md font-medium transition-all ${
                          gender === "Girl" ? 'bg-accent text-white shadow-sm' : 'text-fg-muted hover:text-fg'
                        }`}
                      >
                        Girl
                      </button>
                      <button 
                        onClick={() => setGender("Boy")}
                        disabled={status !== "idle" && status !== "error" && status !== "done"}
                        className={`flex-1 py-1 text-xs rounded-md font-medium transition-all ${
                          gender === "Boy" ? 'bg-accent text-white shadow-sm' : 'text-fg-muted hover:text-fg'
                        }`}
                      >
                        Boy
                      </button>
                    </div>
                  </div>

                  <div>
                    <label className="text-[10px] font-semibold text-fg-muted uppercase tracking-wider mb-1.5 block">Audio Locale</label>
                    <select 
                      value={locale}
                      onChange={(e) => setLocale(e.target.value)}
                      disabled={status !== "idle" && status !== "error" && status !== "done"}
                      className="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-1.5 text-xs text-fg outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/50 appearance-none disabled:opacity-50"
                    >
                      <option value="en" className="bg-bg-deep text-fg">English (en)</option>
                      <option value="jp" className="bg-bg-deep text-fg">Japanese (jp)</option>
                      <option value="kr" className="bg-bg-deep text-fg">Korean (kr)</option>
                      <option value="zh" className="bg-bg-deep text-fg">Chinese (zh)</option>
                    </select>
                  </div>

                  <div>
                    <label className="text-[10px] font-semibold text-fg-muted uppercase tracking-wider mb-1.5 block">Subtitle Mode</label>
                    <select 
                      value={subtitleMode}
                      onChange={(e) => setSubtitleMode(e.target.value as any)}
                      disabled={status !== "idle" && status !== "error" && status !== "done"}
                      className="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-1.5 text-xs text-fg outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/50 appearance-none disabled:opacity-50"
                    >
                      <option value="None" className="bg-bg-deep text-fg">None</option>
                      <option value="Soft-sub" className="bg-bg-deep text-fg">Soft-sub (Muxed track)</option>
                      <option value="Hard-sub" className="bg-bg-deep text-fg">Hard-sub (Burned-in)</option>
                    </select>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-white/[0.06] shrink-0">
                  {status === "error" && (
                    <div className="mb-3 p-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-xs flex items-start gap-2">
                      <AlertCircle className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                      <p className="break-words leading-tight">{log}</p>
                    </div>
                  )}

                  {status === "done" && (
                    <div className="mb-3 p-2.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs flex flex-col gap-2">
                      <div className="flex items-center gap-2">
                        <CheckCircle2 className="w-3.5 h-3.5 shrink-0 text-emerald-400" />
                        <span className="font-medium truncate">{log}</span>
                      </div>
                      <button
                        onClick={handleOpenOutput}
                        className="w-full flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-md bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-300 font-semibold text-xs transition-colors"
                      >
                        <FolderOpen className="w-3.5 h-3.5" />
                        <span>Open Output Folder</span>
                      </button>
                    </div>
                  )}

                  {(status === "extracting" || status === "generating" || status === "rendering") && (
                    <div className="mb-3 text-xs flex flex-col gap-2 text-accent-bright bg-black/40 p-2.5 rounded-lg border border-accent/20">
                      <div className="flex items-center gap-2.5">
                        <Loader2 className="w-3.5 h-3.5 animate-spin shrink-0" />
                        <p className="truncate text-[11px] font-semibold">{log}</p>
                      </div>
                      <div className="bg-bg-deep/50 rounded flex flex-col p-2 h-32 overflow-y-auto custom-scrollbar font-mono text-[9px] text-fg-muted/80 gap-1 mt-1 border border-white/5 select-text cursor-text">
                        {processLogs.length === 0 ? (
                          <span className="italic opacity-50">Waiting for backend...</span>
                        ) : (
                          processLogs.map((l, i) => (
                            <span key={i} className="break-words">{l}</span>
                          ))
                        )}
                        {/* Auto-scroll anchor would go here if we had a ref, but simple flex is ok */}
                      </div>
                    </div>
                  )}

                  <Button 
                    onClick={handleProcess}
                    disabled={status !== "idle" && status !== "error" && status !== "done"}
                    className="w-full py-2.5 text-xs font-semibold"
                  >
                    {status === "idle" || status === "error" || status === "done" ? (
                      <>
                        <Play className="w-3.5 h-3.5" />
                        Start Processing
                      </>
                    ) : (
                      <>
                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        Processing...
                      </>
                    )}
                  </Button>
                </div>
              </SpotlightCard>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </motion.div>
  );
}
