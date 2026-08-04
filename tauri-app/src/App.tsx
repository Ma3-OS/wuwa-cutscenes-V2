import { useState } from "react";
import { Toaster } from "react-hot-toast";
import { Sidebar, TabId } from "./components/layout/Sidebar";
import { BlobBackground } from "./components/ui/BlobBackground";
import { ExtractorView } from "./views/ExtractorView";
import { GalleryView } from "./views/GalleryView";
import { SetupView } from "./views/SetupView";
import { LogView } from "./views/LogView";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, X } from "lucide-react";
import "./index.css";

function App() {
  const [activeTab, setActiveTab] = useState<TabId>("extractor");
  const appWindow = getCurrentWindow();

  return (
    <div className="flex flex-col h-screen w-screen bg-bg-deep text-fg overflow-hidden relative">
      {/* Custom Titlebar */}
      <div 
        data-tauri-drag-region 
        onPointerDown={() => appWindow.startDragging()}
        className="h-9 shrink-0 flex items-center justify-between px-3 z-50 bg-[#0c0d12]/90 border-b border-white/[0.04] select-none"
      >
        <div data-tauri-drag-region className="text-[11px] font-medium text-fg-muted pl-2 flex-1 h-full flex items-center cursor-move pointer-events-none">WuWa Cutscenes V3</div>
        <div className="flex items-center gap-1.5 z-10">
          <button onClick={() => appWindow.minimize()} className="p-1.5 hover:bg-white/10 rounded-md text-fg-subtle hover:text-fg transition-colors">
            <Minus className="w-4 h-4" />
          </button>
          <button onClick={() => appWindow.close()} className="p-1.5 hover:bg-red-500/20 rounded-md text-fg-subtle hover:text-red-400 transition-colors">
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
      
      <div className="flex flex-1 overflow-hidden relative">
        <BlobBackground />
        
        <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
        
        <main className="flex-1 relative z-10 overflow-hidden flex flex-col h-full p-6 pb-4">
            <div className="flex-1 relative overflow-hidden">
            {/* We keep all components mounted but hidden via CSS to preserve state */}
            <div className={activeTab === "extractor" ? "block h-full" : "hidden"}>
              <ExtractorView />
            </div>
            <div className={activeTab === "cutscenes" ? "block h-full" : "hidden"}>
              <GalleryView />
            </div>
            <div className={activeTab === "setup" ? "block h-full" : "hidden"}>
              <SetupView />
            </div>
            <div className={activeTab === "logs" ? "block h-full" : "hidden"}>
              <LogView />
            </div>
          </div>
        </main>
      </div>

      <Toaster
        position="bottom-right"
        toastOptions={{
          style: {
            background: '#0c0d12',
            color: '#EDEDEF',
            border: '1px solid rgba(255,255,255,0.08)',
            borderRadius: '12px',
            fontSize: '13px',
            boxShadow: '0 4px 20px rgba(0,0,0,0.5)',
          },
          success: {
            iconTheme: { primary: '#34d399', secondary: '#0c0d12' },
          },
          error: {
            iconTheme: { primary: '#f87171', secondary: '#0c0d12' },
          },
        }}
      />
    </div>
  );
}

export default App;
