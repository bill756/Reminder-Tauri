import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface WorkTimerProps {
  minutes: number;
  onComplete: () => void;
}

function WorkTimer({ minutes, onComplete }: WorkTimerProps) {
  const [remaining, setRemaining] = useState(minutes * 60);
  const [isWarning, setIsWarning] = useState(false);
  const intervalRef = useRef<number | null>(null);
  const movedRef = useRef(false);

  useEffect(() => {
    intervalRef.current = window.setInterval(() => {
      setRemaining((prev) => {
        if (prev <= 1) {
          if (intervalRef.current) {
            clearInterval(intervalRef.current);
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

  useEffect(() => {
    if (remaining <= 16 && remaining > 0) {
      setIsWarning(true);
      if (!movedRef.current) {
        movedRef.current = true;
        invoke("move_work_window_center").catch(console.error);
      }
    }
  }, [remaining]);

  const mins = Math.floor(remaining / 60);
  const secs = remaining % 60;

  return (
    <div className={`work-timer ${isWarning ? "warning" : ""}`}>
      <div className="time">
        {String(mins).padStart(2, "0")}:{String(secs).padStart(2, "0")}
      </div>
      {isWarning && <div className="warn-text">该休息了！</div>}
    </div>
  );
}

export default WorkTimer;
