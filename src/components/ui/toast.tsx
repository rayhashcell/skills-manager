/**
 * Toast Component
 *
 * A simple toast notification system for displaying messages.
 */

import * as React from "react";
import { X, AlertCircle, CheckCircle2, AlertTriangle, Info } from "lucide-react";
import { cn } from "@/lib/utils";

export type ToastType = "error" | "success" | "warning" | "info";

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
}

interface ToastProps {
  toast: Toast;
  onDismiss: (id: string) => void;
}

const toastIcons: Record<ToastType, React.ReactNode> = {
  error: <AlertCircle className="size-4" />,
  success: <CheckCircle2 className="size-4" />,
  warning: <AlertTriangle className="size-4" />,
  info: <Info className="size-4" />,
};

const toastStyles: Record<ToastType, string> = {
  error: "border-destructive/50 bg-card text-destructive",
  success: "border-success/50 bg-card text-success",
  warning: "border-warning/50 bg-card text-warning-foreground",
  info: "border-primary/50 bg-card text-primary",
};

function ToastItem({ toast, onDismiss }: ToastProps) {
  React.useEffect(() => {
    if (toast.duration !== 0) {
      const timer = setTimeout(() => {
        onDismiss(toast.id);
      }, toast.duration || 5000);
      return () => clearTimeout(timer);
    }
  }, [toast.id, toast.duration, onDismiss]);

  return (
    <div
      className={cn(
        "pointer-events-auto flex w-full max-w-sm items-start gap-3 rounded-lg border p-4 shadow-lg animate-in slide-in-from-top-2 fade-in-0 duration-200",
        toastStyles[toast.type]
      )}
      role="alert"
      aria-live="assertive"
    >
      <span className="shrink-0 mt-0.5">{toastIcons[toast.type]}</span>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium">{toast.title}</p>
        {toast.message && (
          <p className="mt-1 text-xs opacity-80 break-words">{toast.message}</p>
        )}
      </div>
      <button
        onClick={() => onDismiss(toast.id)}
        className="shrink-0 rounded-md p-1 opacity-70 hover:opacity-100 transition-opacity cursor-pointer focus:outline-none focus:ring-2 focus:ring-ring"
        aria-label="Dismiss notification"
      >
        <X className="size-3.5" />
      </button>
    </div>
  );
}

interface ToastContainerProps {
  toasts: Toast[];
  onDismiss: (id: string) => void;
}

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return (
    <div
      className="fixed top-4 right-4 z-50 flex flex-col gap-2 pointer-events-none"
      aria-label="Notifications"
    >
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onDismiss={onDismiss} />
      ))}
    </div>
  );
}

export function useToast() {
  const [toasts, setToasts] = React.useState<Toast[]>([]);

  const addToast = React.useCallback(
    (toast: Omit<Toast, "id">) => {
      const id = `toast-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      setToasts((prev) => [...prev, { ...toast, id }]);
      return id;
    },
    []
  );

  const dismissToast = React.useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const showError = React.useCallback(
    (title: string, message?: string) => {
      return addToast({ type: "error", title, message });
    },
    [addToast]
  );

  const showSuccess = React.useCallback(
    (title: string, message?: string) => {
      return addToast({ type: "success", title, message });
    },
    [addToast]
  );

  const showWarning = React.useCallback(
    (title: string, message?: string) => {
      return addToast({ type: "warning", title, message });
    },
    [addToast]
  );

  const showInfo = React.useCallback(
    (title: string, message?: string) => {
      return addToast({ type: "info", title, message });
    },
    [addToast]
  );

  return {
    toasts,
    addToast,
    dismissToast,
    showError,
    showSuccess,
    showWarning,
    showInfo,
  };
}

export { ToastItem };
