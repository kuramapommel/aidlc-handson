import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from 'react';

type ToastKind = 'success' | 'info' | 'error';

interface Toast {
  id: number;
  kind: ToastKind;
  message: string;
}

interface ToastValue {
  show: (message: string, kind?: ToastKind) => void;
}

const ToastContext = createContext<ToastValue | null>(null);
const DURATION_MS = 3200;
const ICONS: Record<ToastKind, string> = { success: '✓', info: 'ℹ', error: '!' };

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(1);

  const dismiss = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const show = useCallback(
    (message: string, kind: ToastKind = 'success') => {
      const id = nextId.current++;
      // 同時表示は最大 3 件まで
      setToasts((prev) => [...prev.slice(-2), { id, kind, message }]);
      window.setTimeout(() => dismiss(id), DURATION_MS);
    },
    [dismiss],
  );

  const value = useMemo(() => ({ show }), [show]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div className="toast-region" role="status" aria-live="polite">
        {toasts.map((t) => (
          <div key={t.id} className={`toast toast--${t.kind}`}>
            <span className="toast__icon" aria-hidden="true">
              {ICONS[t.kind]}
            </span>
            <span className="toast__message">{t.message}</span>
            <button type="button" className="toast__close" aria-label="通知を閉じる" onClick={() => dismiss(t.id)}>
              ×
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast(): ToastValue {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error('useToast は ToastProvider の内側で使用してください');
  return ctx;
}
