import { motion } from "framer-motion";
import { Construction } from "lucide-react";
import { SpotlightCard } from "../components/ui/SpotlightCard";

interface PlaceholderViewProps {
  title: string;
  description: string;
}

export function PlaceholderView({ title, description }: PlaceholderViewProps) {
  return (
    <motion.div 
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
      className="flex flex-col items-center justify-center h-full pt-20"
    >
      <SpotlightCard className="w-full max-w-md p-10 flex flex-col items-center text-center">
        <div className="w-20 h-20 rounded-3xl bg-accent/10 border border-accent/20 flex items-center justify-center mb-6 shadow-inner-glow">
          <Construction className="w-10 h-10 text-accent-bright" />
        </div>
        
        <h1 className="text-3xl font-semibold tracking-tight text-fg mb-3">{title}</h1>
        <p className="text-fg-muted mb-8">{description}</p>
        
        <div className="px-4 py-2 rounded-full bg-white/[0.04] border border-white/[0.06] text-xs font-mono tracking-widest uppercase text-fg-subtle">
          Module in Development
        </div>
      </SpotlightCard>
    </motion.div>
  );
}
