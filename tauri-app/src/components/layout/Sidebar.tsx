import { motion } from "framer-motion";
import { clsx } from "clsx";
import { PackageSearch, Clapperboard, FolderOpen, Film, Settings2, ScrollText } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

export type TabId = "extractor" | "cutscenes" | "setup" | "logs";

interface NavItem {
  id: TabId;
  label: string;
  icon: React.ElementType;
}

const NAV_ITEMS: NavItem[] = [
  { id: "extractor", label: "Mount Paks", icon: PackageSearch },
  { id: "cutscenes", label: "Cutscene Gallery", icon: Clapperboard },
  { id: "setup", label: "Setup & Config", icon: Settings2 },
  { id: "logs", label: "Activity Log", icon: ScrollText },
];

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const handleOpenOutput = async () => {
    try {
      await invoke("open_output_dir");
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <aside className="w-[280px] border-r border-white/[0.08] bg-[#0c0d12]/90 backdrop-blur-2xl flex flex-col pt-6 pb-6 pl-6 pr-4 z-20 shrink-0 select-none">
      {/* App Branding Header */}
      <div className="mb-8 px-2 flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/20">
            <Film className="w-4 h-4 text-white" />
          </div>
          <div>
            <h2 className="text-base font-bold tracking-tight text-white leading-none">
              WuWa Cutscene
            </h2>
            <span className="text-[10px] font-medium text-fg-muted tracking-wider uppercase mt-1 block">
              Studio Pro
            </span>
          </div>
        </div>
      </div>

      {/* Primary Navigation */}
      <nav className="flex flex-col gap-1.5 flex-1">
        <div className="px-2 pb-2 text-[10px] font-semibold text-fg-muted uppercase tracking-widest opacity-60">
          Navigation
        </div>
        {NAV_ITEMS.map((item) => {
          const isActive = activeTab === item.id;
          const Icon = item.icon;
          
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              className={clsx(
                "group relative flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 outline-none",
                isActive ? "text-white font-semibold" : "text-fg-muted hover:text-fg hover:bg-white/[0.04]"
              )}
            >
              {isActive && (
                <motion.div
                  layoutId="sidebar-active"
                  className="absolute inset-0 rounded-xl bg-gradient-to-r from-accent/20 to-purple-500/10 border border-accent/30 shadow-sm"
                  initial={false}
                  transition={{ type: "spring", stiffness: 400, damping: 30 }}
                />
              )}
              
              <Icon 
                className={clsx(
                  "w-4 h-4 relative z-10 transition-colors", 
                  isActive ? "text-accent-bright" : "text-fg-muted group-hover:text-fg"
                )} 
              />
              <span className="relative z-10">{item.label}</span>
            </button>
          );
        })}
      </nav>
      
      {/* Quick Action: Open Output Directory */}
      <div className="mt-auto pt-4 border-t border-white/[0.06] flex flex-col gap-3">
        <button
          onClick={handleOpenOutput}
          className="w-full flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-xs font-medium bg-white/[0.04] border border-white/[0.08] text-fg-muted hover:text-white hover:bg-white/[0.08] hover:border-white/20 transition-all duration-200 group"
          title="Open local output folder in File Explorer"
        >
          <FolderOpen className="w-4 h-4 text-accent group-hover:scale-110 transition-transform" />
          <span>Output Folder</span>
        </button>

        <div className="px-2 flex items-center justify-between text-[11px] text-fg-subtle font-mono opacity-50">
          <span>v1.0.0</span>
          <span>Pro Edition</span>
        </div>
      </div>
    </aside>
  );
}
