import { useEffect, useRef } from 'react';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  showCloseButton?: boolean;
}

export default function Modal({
  isOpen,
  onClose,
  title,
  children,
  size = 'md',
  showCloseButton = true
}: ModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };

    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      document.body.style.overflow = 'hidden';
    }

    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = 'unset';
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const sizeClasses = {
    sm: 'max-w-md',
    md: 'max-w-2xl',
    lg: 'max-w-4xl',
    xl: 'max-w-6xl',
  };

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-in fade-in duration-200"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={modalRef}
        className={`
          ${sizeClasses[size]} w-full
          bg-gradient-to-br from-stone-900 to-stone-800
          border-2 border-adventure-600/40 rounded-2xl shadow-2xl shadow-adventure-900/30
          animate-in zoom-in-95 duration-200
          max-h-[90vh] flex flex-col
        `}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-adventure-600/30 bg-gradient-to-r from-stone-800/50 to-stone-900/50 flex-shrink-0">
          <h2 className="text-2xl font-bold text-white">{title}</h2>
          {showCloseButton && (
            <button
              onClick={onClose}
              className="text-adventure-300 hover:text-white transition-all duration-200 p-2 rounded-lg hover:bg-adventure-900/30 hover:scale-110"
              aria-label="Close modal"
            >
              <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>

        {/* Content - Scrollable */}
        <div className="p-6 overflow-y-auto flex-1">
          {children}
        </div>
      </div>
    </div>
  );
}
