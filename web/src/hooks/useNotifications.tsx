import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import type { WebSocketMessage } from "../../../src/types";
import { useWebSocket } from "./useWebSocket";
import { dedupeNotifications, notificationFromRealtimeMessage, type AppNotification } from "../utils/notifications";

interface NotificationsContextType {
  notifications: AppNotification[];
  unreadCount: number;
  dismissNotification: (id: string) => void;
  markAllRead: () => void;
  markNotificationRead: (id: string) => void;
}

const NotificationsContext = createContext<NotificationsContextType | null>(null);

export function useNotifications() {
  const context = useContext(NotificationsContext);
  if (!context) {
    throw new Error("useNotifications must be used within NotificationsProvider");
  }
  return context;
}

export function NotificationsProvider({ children }: { children: ReactNode }) {
  const { lastMessage } = useWebSocket();
  const [notifications, setNotifications] = useState<AppNotification[]>([]);

  useEffect(() => {
    if (!lastMessage || typeof lastMessage === "string") {
      return;
    }

    const nextNotification = notificationFromRealtimeMessage(lastMessage as WebSocketMessage);
    if (!nextNotification) {
      return;
    }

    setNotifications((current) =>
      dedupeNotifications([{ ...nextNotification, read: false }, ...current]).slice(0, 20),
    );

    // Auto-dismiss toast after 8 seconds
    const timerId = window.setTimeout(() => {
      setNotifications((current) =>
        current.map((n) => (n.id === nextNotification.id ? { ...n, read: true } : n)),
      );
    }, 8000);

    return () => window.clearTimeout(timerId);
  }, [lastMessage]);

  const value = useMemo<NotificationsContextType>(
    () => ({
      notifications,
      unreadCount: notifications.filter((notification) => !notification.read).length,
      dismissNotification: (id) => {
        setNotifications((current) => current.filter((notification) => notification.id !== id));
      },
      markAllRead: () => {
        setNotifications((current) =>
          current.map((notification) => ({
            ...notification,
            read: true,
          })),
        );
      },
      markNotificationRead: (id) => {
        setNotifications((current) =>
          current.map((notification) =>
            notification.id === id
              ? {
                  ...notification,
                  read: true,
                }
              : notification,
          ),
        );
      },
    }),
    [notifications],
  );

  return <NotificationsContext.Provider value={value}>{children}</NotificationsContext.Provider>;
}
