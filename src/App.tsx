import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface UiElement {
  kind: string;
  label: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

interface GuidanceStep {
  step: number;
  instruction: string;
  element: UiElement | null;
}

interface AnalysisResponse {
  summary: string;
  steps: GuidanceStep[];
}

interface Settings {
  api_key: string;
  model: string;
}

const MODELS = [
  { value: "anthropic/claude-sonnet-4", label: "Claude Sonnet 4" },
  { value: "anthropic/claude-haiku-4-5", label: "Claude Haiku 4.5" },
  { value: "openai/gpt-4o", label: "GPT-4o" },
  { value: "openai/gpt-4o-mini", label: "GPT-4o Mini" },
  { value: "google/gemini-2.5-flash", label: "Gemini 2.5 Flash" },
];

function App() {
  const [question, setQuestion] = useState("");
  const [status, setStatus] = useState<
    "idle" | "capturing" | "analyzing" | "error"
  >("idle");
  const [errorMsg, setErrorMsg] = useState("");
  const [showSettings, setShowSettings] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("anthropic/claude-sonnet-4");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Load settings on mount
    invoke<Settings>("get_settings").then((s) => {
      setApiKey(s.api_key);
      setModel(s.model);
    });
  }, []);

  useEffect(() => {
    const focusInput = () => {
      if (!showSettings) {
        inputRef.current?.focus();
        setStatus("idle");
        setQuestion("");
        setErrorMsg("");
      }
    };
    window.addEventListener("focus", focusInput);
    focusInput();
    return () => window.removeEventListener("focus", focusInput);
  }, [showSettings]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (showSettings) {
          setShowSettings(false);
        } else {
          invoke("hide_window");
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showSettings]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const q = question.trim();
    if (!q) return;

    setStatus("capturing");
    setErrorMsg("");
    try {
      const result = await invoke<AnalysisResponse>("submit_question", {
        question: q,
      });
      console.log("Analysis result:", result);
      setStatus("idle");
      setQuestion("");
      // TODO: DW-6 will render the overlay with result.steps
    } catch (err) {
      console.error("Failed:", err);
      setErrorMsg(String(err));
      setStatus("error");
      // Show error briefly, then reset
      setTimeout(() => {
        setStatus("idle");
        setErrorMsg("");
      }, 4000);
    }
  };

  const handleSaveSettings = async () => {
    await invoke("save_settings", { apiKey, model });
    setShowSettings(false);
    inputRef.current?.focus();
  };

  if (showSettings) {
    return (
      <div className="input-bar settings-bar" data-tauri-drag-region>
        <div className="settings-form">
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="OpenRouter API key"
            className="input-field"
            autoFocus
          />
          <select
            value={model}
            onChange={(e) => setModel(e.target.value)}
            className="model-select"
          >
            {MODELS.map((m) => (
              <option key={m.value} value={m.value}>
                {m.label}
              </option>
            ))}
          </select>
          <button onClick={handleSaveSettings} className="submit-btn">
            Save
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="input-bar" data-tauri-drag-region>
      {status === "capturing" || status === "analyzing" ? (
        <div className="status-msg">
          {status === "capturing" ? "Capturing screen..." : "Analyzing..."}
        </div>
      ) : status === "error" ? (
        <div className="status-msg error-msg">{errorMsg}</div>
      ) : (
        <form onSubmit={handleSubmit} className="input-form">
          <button
            type="button"
            onClick={() => setShowSettings(true)}
            className="settings-btn"
            title="Settings"
          >
            &#9881;
          </button>
          <input
            ref={inputRef}
            type="text"
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            placeholder="Ask ShowMe anything... (Esc to dismiss)"
            className="input-field"
            autoFocus
          />
          <button
            type="submit"
            className="submit-btn"
            disabled={!question.trim()}
          >
            Ask
          </button>
        </form>
      )}
    </div>
  );
}

export default App;
