import { Button } from "@/components/ui/button";
import { useWebSocket } from "@/contexts/WebSocketContext";
import { Wifi, WifiOff } from "lucide-react";
import { useEffect } from "react";
import { toast } from "sonner";

export default function ConnectionStatus() {
  const { isConnected, connectionState, connect, disconnect } = useWebSocket();

  useEffect(() => {
    // 自动尝试连接
    if (!isConnected && connectionState === 'disconnected') {
      connect().catch((error) => {
        console.error('Auto-connect failed:', error);
      });
    }
  }, []);

  useEffect(() => {
    if (isConnected) {
      toast.success('已连接到引擎', {
        description: '实时通信已建立',
        duration: 2000,
      });
    } else if (connectionState === 'disconnected') {
      toast.error('引擎连接断开', {
        description: '正在尝试重新连接...',
        duration: 3000,
      });
    }
  }, [isConnected, connectionState]);

  const handleToggleConnection = () => {
    if (isConnected) {
      disconnect();
    } else {
      connect();
    }
  };

  return (
    <Button
      size="sm"
      variant="ghost"
      className="h-8 gap-2"
      onClick={handleToggleConnection}
    >
      {isConnected ? (
        <>
          <Wifi className="w-4 h-4 text-green-500" />
          <span className="text-xs">已连接</span>
        </>
      ) : (
        <>
          <WifiOff className="w-4 h-4 text-red-500" />
          <span className="text-xs">{connectionState === 'connecting' ? '连接中' : '未连接'}</span>
        </>
      )}
    </Button>
  );
}
