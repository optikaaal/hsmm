interface StatusPillProps {
  status: 'enabled' | 'disabled' | 'update' | 'installing' | 'error';
  label?: string;
  showDot?: boolean;
}

export default function StatusPill({ status, label, showDot = true }: StatusPillProps) {
  const configs = {
    enabled: {
      color: 'bg-green-500/20 text-green-300 border-green-500/40',
      dotColor: 'bg-green-500',
      defaultLabel: 'Enabled',
    },
    disabled: {
      color: 'bg-gray-500/20 text-gray-300 border-gray-500/40',
      dotColor: 'bg-gray-500',
      defaultLabel: 'Disabled',
    },
    update: {
      color: 'bg-diamond-500/20 text-diamond-300 border-diamond-500/40',
      dotColor: 'bg-diamond-500',
      defaultLabel: 'Update Available',
    },
    installing: {
      color: 'bg-adventure-500/20 text-adventure-300 border-adventure-500/40',
      dotColor: 'bg-adventure-500',
      defaultLabel: 'Installing...',
    },
    error: {
      color: 'bg-red-500/20 text-red-300 border-red-500/40',
      dotColor: 'bg-red-500',
      defaultLabel: 'Error',
    },
  };

  const config = configs[status];

  return (
    <span
      className={`
        inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full border text-xs font-medium
        ${config.color}
      `}
    >
      {showDot && (
        <span className={`w-1.5 h-1.5 rounded-full ${config.dotColor}`}></span>
      )}
      {label || config.defaultLabel}
    </span>
  );
}
