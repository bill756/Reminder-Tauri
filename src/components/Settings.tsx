import { useState } from "react";

interface SettingsProps {
  onStart: (work: number, rest: number, block: boolean) => void;
  defaultWork?: number;
  defaultRest?: number;
  defaultBlock?: boolean;
}

function Settings({ onStart, defaultWork = 60, defaultRest = 5, defaultBlock = true }: SettingsProps) {
  const [workTime, setWorkTime] = useState(defaultWork);
  const [restTime, setRestTime] = useState(defaultRest);
  const [inputBlock, setInputBlock] = useState(defaultBlock);

  const handleStart = () => {
    onStart(workTime, restTime, inputBlock);
  };

  return (
    <div className="settings">
      <h1>久坐提醒</h1>

      <div className="form-group">
        <label>工作时间（分钟）</label>
        <input
          type="number"
          value={workTime}
          onChange={(e) => setWorkTime(Math.max(1, parseInt(e.target.value) || 1))}
          min="1"
          max="120"
        />
      </div>

      <div className="form-group">
        <label>休息时间（分钟）</label>
        <input
          type="number"
          value={restTime}
          onChange={(e) => setRestTime(Math.max(1, parseInt(e.target.value) || 1))}
          min="1"
          max="30"
        />
      </div>

      <div className="form-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={inputBlock}
            onChange={(e) => setInputBlock(e.target.checked)}
          />
          锁定键盘鼠标（需管理员权限）
        </label>
      </div>

      <button className="btn-start" onClick={handleStart}>
        开始
      </button>
    </div>
  );
}

export default Settings;
