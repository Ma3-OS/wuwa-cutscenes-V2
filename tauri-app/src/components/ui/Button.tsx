import { motion, HTMLMotionProps } from "framer-motion";
import { ReactNode } from "react";
import { twMerge } from "tailwind-merge";
import { clsx, type ClassValue } from "clsx";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface ButtonProps extends HTMLMotionProps<"button"> {
  children: ReactNode;
  variant?: "primary" | "secondary" | "ghost";
  className?: string;
}

export function Button({ 
  children, 
  variant = "primary", 
  className, 
  disabled, 
  ...props 
}: ButtonProps) {
  
  const baseStyles = "relative inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base disabled:pointer-events-none disabled:opacity-50 overflow-hidden";
  
  const variants = {
    primary: "bg-accent text-white shadow-btn-glow hover:bg-accent-bright",
    secondary: "bg-white/[0.05] text-fg border border-white/10 hover:bg-white/[0.08] shadow-inner-glow",
    ghost: "bg-transparent text-fg-muted hover:bg-white/[0.05] hover:text-fg"
  };

  return (
    <motion.button
      whileHover={{ y: -1 }}
      whileTap={{ scale: 0.98 }}
      transition={{ type: "tween", ease: [0.16, 1, 0.3, 1], duration: 0.2 }}
      className={cn(baseStyles, variants[variant], className)}
      disabled={disabled}
      {...props}
    >
      {/* Primary variant shine effect on hover */}
      {variant === "primary" && !disabled && (
        <span className="absolute inset-0 z-0 bg-gradient-to-tr from-transparent via-white/20 to-transparent opacity-0 hover:opacity-100 transition-opacity duration-300" />
      )}
      <span className="relative z-10 flex items-center gap-2">{children}</span>
    </motion.button>
  );
}
