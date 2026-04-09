import { type FC, useEffect, useState } from "react";
import { WifiOff } from "lucide-react";

export const OfflineBanner: FC = () => {
  const [isOffline, setIsOffline] = useState(
    typeof navigator !== "undefined" ? !navigator.onLine : false,
  );

  useEffect(() => {
    const handleOffline = () => setIsOffline(true);
    const handleOnline = () => setIsOffline(false);

    window.addEventListener("offline", handleOffline);
    window.addEventListener("online", handleOnline);

    return () => {
      window.removeEventListener("offline", handleOffline);
      window.removeEventListener("online", handleOnline);
    };
  }, []);

  if (!isOffline) return null;

  return (
    <div
      className="flex items-center justify-center gap-2 bg-warning px-4 py-1.5 text-warning-foreground"
      role="alert"
    >
      <WifiOff size={14} />
      <span className="font-mono text-[11px] font-medium">
        You are offline — some features may be unavailable
      </span>
    </div>
  );
};
