interface BadgeProps {
  children: React.ReactNode;
  variant?: 'default' | 'success' | 'error' | 'warning' | 'info' | 'adventure';
  size?: 'sm' | 'md' | 'lg';
}

export default function Badge({ children, variant = 'default', size = 'md' }: BadgeProps) {
  const variants = {
    default: 'bg-stone-500/30 text-stone-200 border-stone-500/50',
    success: 'bg-green-500/30 text-green-300 border-green-500/50',
    error: 'bg-red-500/30 text-red-300 border-red-500/50',
    warning: 'bg-lava-500/30 text-lava-300 border-lava-500/50',
    info: 'bg-diamond-500/30 text-diamond-300 border-diamond-500/50',
    adventure: 'bg-adventure-500/30 text-adventure-300 border-adventure-500/50',
  };

  const sizes = {
    sm: 'px-1.5 py-0.5 text-xs',
    md: 'px-2 py-0.5 text-xs',
    lg: 'px-3 py-1 text-sm',
  };

  return (
    <span
      className={`
        inline-flex items-center rounded border font-medium
        ${variants[variant]} ${sizes[size]}
      `}
    >
      {children}
    </span>
  );
}
