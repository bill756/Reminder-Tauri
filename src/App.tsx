import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Settings from "./components/Settings";
import WorkTimer from "./components/WorkTimer";
import RestOverlay from "./components/RestOverlay";

type Mode = "settings" | "work" | "rest";

function App() {
  const [mode, setMode] = useState<Mode>("settings");
  const [workMinutes, setWorkMinutes] = useState(60);
  const [restMinutes, setRestMinutes] = useState(5);
  const [inputBlock, setInputBlock] = useState(true);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const urlMode = params.get("mode");
    const work = params.get("work");
    const rest = params.get("rest");
    const block = params.get("block");

    if (urlMode === "work") {
      setMode("work");
      if (work) setWorkMinutes(parseInt(work));
      if (rest) setRestMinutes(parseInt(rest));
      if (block) setInputBlock(block === "1");
    } else if (urlMode === "rest") {
      setMode("rest");
      if (work) setWorkMinutes(parseInt(work));
      if (rest) setRestMinutes(parseInt(rest));
      if (block) setInputBlock(block === "1");
    }
  }, []);

  const handleStart = async (work: number, rest: number, block: boolean) => {
    setWorkMinutes(work);
    setRestMinutes(rest);
    setInputBlock(block);

    await invoke("hide_main_window");
    await invoke("create_work_window", { workMinutes: work, restMinutes: rest, inputBlock: block });
  };

  const handleWorkComplete = async () => {
    await invoke("create_rest_window", {
      workMinutes,
      restMinutes,
      inputBlock,
    });
    setTimeout(() => {
      invoke("close_window", { label: "work" });
    }, 500);
  };

  const handleRestComplete = async () => {
    await invoke("create_work_window", { workMinutes, restMinutes, inputBlock });
    setTimeout(() => {
      invoke("close_all_rest_windows");
    }, 500);
  };

  if (mode === "work") {
    return <WorkTimer minutes={workMinutes} onComplete={handleWorkComplete} />;
  }

  if (mode === "rest") {
    return (
      <RestOverlay
        workMinutes={workMinutes}
        restMinutes={restMinutes}
        inputBlock={inputBlock}
        onComplete={handleRestComplete}
      />
    );
  }

  return (
    <Settings
      onStart={handleStart}
      defaultWork={workMinutes}
      defaultRest={restMinutes}
      defaultBlock={inputBlock}
    />
  );
}

export default App;
