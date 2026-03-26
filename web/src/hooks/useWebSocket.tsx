import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useAuth } from './useAuth';
import type { Node } from './useNodes';
import type { WebSocketMessage } from '../../../src/types';

interface WebSocketContextType {
  connected: boolean;
  lastMessage: WebSocketMessage | string | null;
}

const WebSocketContext = createContext<WebSocketContextType>({
  connected: false,
  lastMessage: null,
});

export function useWebSocket() {
  return useContext(WebSocketContext);
}

interface WebSocketProviderProps {
  children: ReactNode;
}

export function WebSocketProvider({ children }: WebSocketProviderProps) {
  const queryClient = useQueryClient();
  const { token } = useAuth();
  const [connected, setConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | string | null>(null);
  const reconnectTimerRef = useRef<number | null>(null);
  const closedByEffectRef = useRef(false);
  const retryAttemptRef = useRef(0);

  useEffect(() => {
    closedByEffectRef.current = false;
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    let ws: WebSocket | null = null;

    const clearReconnect = () => {
      if (reconnectTimerRef.current !== null) {
        window.clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
    };

    const connect = () => {
      clearReconnect();
      const token = localStorage.getItem('token');
      if (!token) {
        // No auth token — delay and retry after login
        reconnectTimerRef.current = window.setTimeout(connect, 2000);
        return;
      }
      ws = new WebSocket(`${protocol}//${window.location.host}/ws?token=${encodeURIComponent(token)}`);

      ws.onopen = () => {
        retryAttemptRef.current = 0;
        setConnected(true);
      };

      ws.onclose = () => {
        setConnected(false);
        if (closedByEffectRef.current) {
          return;
        }

        const nextAttempt = retryAttemptRef.current + 1;
        retryAttemptRef.current = nextAttempt;
        const delay = Math.min(5000, 500 * 2 ** (nextAttempt - 1));
        reconnectTimerRef.current = window.setTimeout(connect, delay);
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as WebSocketMessage;
          setLastMessage(data);
        } catch {
          setLastMessage(event.data);
        }
      };

      ws.onerror = () => {
        // Let onclose handle reconnects; avoid noisy initial connection logs.
      };
    };

    connect();

    return () => {
      closedByEffectRef.current = true;
      clearReconnect();
      ws?.close();
    };
  }, [token]);

  useEffect(() => {
    if (!lastMessage || typeof lastMessage === 'string' || !lastMessage.nodeId) {
      return;
    }

    if (lastMessage.type === 'status') {
      const { nodeId } = lastMessage;
      const statusData = lastMessage.data as { status: Node['process']['status'] };

      queryClient.setQueryData<Node[]>(['nodes'], (current) =>
        current?.map((node) =>
          node.id === nodeId
            ? {
                ...node,
                process: {
                  ...node.process,
                  status: statusData.status,
                },
              }
            : node,
        ) ?? current,
      );

      queryClient.setQueryData<Node>(['nodes', nodeId], (current) =>
        current
          ? {
              ...current,
              process: {
                ...current.process,
                status: statusData.status,
              },
            }
          : current,
      );
    }

    if (lastMessage.type === 'metrics') {
      const { nodeId } = lastMessage;
      const metricsData = lastMessage.data as Node['metrics'];

      queryClient.setQueryData<Node[]>(['nodes'], (current) =>
        current?.map((node) =>
          node.id === nodeId
            ? {
                ...node,
                metrics: metricsData,
              }
            : node,
        ) ?? current,
      );

      queryClient.setQueryData<Node>(['nodes', nodeId], (current) =>
        current
          ? {
              ...current,
              metrics: metricsData,
            }
          : current,
      );
    }
  }, [lastMessage, queryClient]);

  return (
    <WebSocketContext.Provider value={{ connected, lastMessage }}>
      {children}
    </WebSocketContext.Provider>
  );
}
