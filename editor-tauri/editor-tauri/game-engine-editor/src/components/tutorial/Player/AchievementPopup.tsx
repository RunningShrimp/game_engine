import React, { useEffect } from 'react';
import { Trophy, X } from 'lucide-react';

interface AchievementPopupProps {
  message: string;
  duration?: number;
  onClose?: () => void;
}

const AchievementPopup: React.FC<AchievementPopupProps> = ({
  message,
  duration = 3000,
  onClose
}) => {
  useEffect(() => {
    const timer = setTimeout(() => {
      onClose?.();
    }, duration);

    return () => clearTimeout(timer);
  }, [duration, onClose]);

  return (
    <div className="fixed top-4 right-4 z-50 animate-slide-in">
      <div className="bg-gradient-to-r from-yellow-400 to-orange-500 dark:from-yellow-600 dark:to-orange-600 text-white px-6 py-4 rounded-lg shadow-lg flex items-center gap-3 min-w-80">
        <div className="bg-white/20 rounded-full p-2">
          <Trophy className="w-6 h-6" />
        </div>
        <div className="flex-1">
          <p className="font-semibold">🎉 {message}</p>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="p-1 hover:bg-white/20 rounded-full transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        )}
      </div>

      <style>{`
        @keyframes slide-in {
          from {
            transform: translateX(100%);
            opacity: 0;
          }
          to {
            transform: translateX(0);
            opacity: 1;
          }
        }
        .animate-slide-in {
          animation: slide-in 0.3s ease-out;
        }
      `}</style>
    </div>
  );
};

export default AchievementPopup;
