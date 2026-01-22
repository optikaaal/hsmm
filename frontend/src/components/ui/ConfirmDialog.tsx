import Modal from './Modal';

interface ConfirmDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: 'danger' | 'warning' | 'info';
}

export default function ConfirmDialog({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  variant = 'info',
}: ConfirmDialogProps) {
  const variantStyles = {
    danger: 'bg-red-600 hover:bg-red-700',
    warning: 'bg-lava-600 hover:bg-lava-700',
    info: 'bg-adventure-600 hover:bg-adventure-700',
  };

  const handleConfirm = () => {
    onConfirm();
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={title} size="sm">
      <div className="space-y-6">
        <p className="text-stone-300 leading-relaxed">{message}</p>

        <div className="flex gap-3 justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-stone-600 hover:bg-stone-700 text-white rounded-lg font-medium transition-colors"
          >
            {cancelText}
          </button>
          <button
            onClick={handleConfirm}
            className={`px-4 py-2 ${variantStyles[variant]} text-white rounded-lg font-medium transition-colors`}
          >
            {confirmText}
          </button>
        </div>
      </div>
    </Modal>
  );
}
