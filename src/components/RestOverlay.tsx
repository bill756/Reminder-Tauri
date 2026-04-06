import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface RestOverlayProps {
  workMinutes: number;
  restMinutes: number;
  inputBlock: boolean;
  onComplete: () => void;
}

function RestOverlay({ workMinutes, restMinutes, inputBlock, onComplete }: RestOverlayProps) {
  const [remaining, setRemaining] = useState(restMinutes * 60);
  const [isAdmin, setIsAdmin] = useState(false);
  const inputBlockedRef = useRef(false);
  const intervalRef = useRef<number | null>(null);

  // Check admin and block input on mount
  useEffect(() => {
    const init = async () => {
      const admin = await invoke<boolean>("is_admin");
      setIsAdmin(admin);
      if (admin && inputBlock) {
        await invoke("block_input", { block: true });
        inputBlockedRef.current = true;
      }
    };
    init();
  }, [inputBlock]);

  // Countdown timer
  useEffect(() => {
    intervalRef.current = window.setInterval(() => {
      setRemaining((prev) => {
        if (prev <= 1) {
          if (intervalRef.current) {
            clearInterval(intervalRef.current);
          }
          if (inputBlockedRef.current) {
            invoke("block_input", { block: false });
            inputBlockedRef.current = false;
          }
          onComplete();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [onComplete]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (inputBlockedRef.current) {
        invoke("block_input", { block: false });
      }
    };
  }, []);

  const mins = Math.floor(remaining / 60);
  const secs = remaining % 60;

  // Small window mode (no block)
  if (!inputBlock) {
    return (
      <div className="rest-timer-window">
        <div className="rest-timer-content">
          <div className="rest-timer-label">休息中</div>
          <div className="rest-timer-time">
            {String(mins).padStart(2, "0")}:{String(secs).padStart(2, "0")}
          </div>
          <button className="rest-timer-close" onClick={onComplete}>
            结束休息
          </button>
        </div>
      </div>
    );
  }

  // Fullscreen overlay mode (with block)
  return (
    <div className="rest-overlay">
      <div className="rest-content">
        <h2>您已久坐 {workMinutes} 分钟了</h2>
        {isAdmin && <p className="warning">键盘和鼠标被锁定，站起来活动下！</p>}
        {!isAdmin && <p className="warning">请用管理员权限运行以锁定键盘</p>}

        <div className="rest-timer">
          {String(mins).padStart(2, "0")}:{String(secs).padStart(2, "0")}
        </div>
      </div>
    </div>
  );
}

export default RestOverlay;