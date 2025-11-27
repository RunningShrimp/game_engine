import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { websocketService, MessageType, MessageHandler } from '@/lib/websocket';

interface WebSocketContextType {
  isConnected: boolean;
  connectionState: string;
  connect: () => Promise<void>;
  disconnect: () => void;
  send: (type: MessageType, data: any) => void;
  subscribe: (type: MessageType, handler: MessageHandler) => () => void;
}

const WebSocketContext = createContext<WebSocketContextType | undefined>(undefined);

export function WebSocketProvider({ children }: { children: ReactNode }) {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionState, setConnectionState] = useState('disconnected');

  useEffect(() => {
    // 定期更新连接状态
    const interval = setInterval(() => {
      setIsConnected(websocketService.isConnected);
      setConnectionState(websocketService.connectionState);
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const connect = async () => {
    try {
      await websocketService.connect();
      setIsConnected(true);
    } catch (error) {
      console.error('Failed to connect:', error);
      setIsConnected(false);
    }
  };

  const disconnect = () => {
    websocketService.disconnect();
    setIsConnected(false);
  };

  const send = (type: MessageType, data: any) => {
    websocketService.send(type, data);
  };

  const subscribe = (type: MessageType, handler: MessageHandler) => {
    return websocketService.on(type, handler);
  };

  return (
    <WebSocketContext.Provider
      value={{
        isConnected,
        connectionState,
        connect,
        disconnect,
        send,
        subscribe,
      }}
    >
      {children}
    </WebSocketContext.Provider>
  );
}

export function useWebSocket() {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocket must be used within WebSocketProvider');
  }
  return context;
}
