import React, { createContext, useContext, useState, ReactNode, useCallback } from 'react';
import ToastContainer, { Toast, ToastType } from '@/components/UI/Toast';

interface ToastContextValue {
  showToast: (type: ToastType, title: string, message?: string, options?: {
    duration?: number;
    action?: { label: string; onClick: () => void };
  }) => void;
  success: (title: string, message?: string) => void;
  error: (title: string, message?: string) => void;
  warning: (title: string, message?: string) => void;
  info: (title: string, message?: string) => void;
  removeToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

interface ToastProviderProps {
  children: ReactNode;
}

export const ToastProvider: React.FC<ToastProviderProps> = ({ children }) => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const showToast = (
    type: ToastType, 
    title: string, 
    message?: string, 
    options?: {
      duration?: number;
      action?: { label: string; onClick: () => void };
    }
  ) => {
    const id = Math.random().toString(36).substring(2, 9);
    const newToast: Toast = {
      id,
      type,
      title,
      message,
      duration: options?.duration,
      action: options?.action,
    };

    setToasts((prev) => [...prev, newToast]);
  };

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  const success = (title: string, message?: string) => {
    showToast('success', title, message, { duration: 5000 });
  };

  const error = (title: string, message?: string) => {
    showToast('error', title, message, { duration: 5000 });
  };

  const warning = (title: string, message?: string) => {
    showToast('warning', title, message, { duration: 5000 });
  };

  const info = (title: string, message?: string) => {
    showToast('info', title, message, { duration: 5000 });
  };

  const value: ToastContextValue = {
    showToast,
    success,
    error,
    warning,
    info,
    removeToast,
  };

  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastContainer toasts={toasts} onClose={removeToast} />
    </ToastContext.Provider>
  );
};

export const useToast = (): ToastContextValue => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
};