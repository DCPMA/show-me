import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [question, setQuestion] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Focus input whenever the window becomes visible
    const focusInput = () => {
      inputRef.current?.focus();
    };
    window.addEventListener("focus", focusInput);
    // Focus on mount too
    focusInput();
    return () => window.removeEventListener("focus", focusInput);
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        invoke("hide_window");
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const q = question.trim();
    if (!q) return;
    setQuestion("");
    await invoke("submit_question", { question: q });
  };

  return (
    <div className="input-bar" data-tauri-drag-region>
      <form onSubmit={handleSubmit} className="input-form">
        <input
          ref={inputRef}
          type="text"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="Ask ShowMe anything... (Esc to dismiss)"
          className="input-field"
          autoFocus
        />
        <button type="submit" className="submit-btn" disabled={!question.trim()}>
          Ask
        </button>
      </form>
    </div>
  );
}

export default App;
