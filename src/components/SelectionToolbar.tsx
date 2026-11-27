import { useState, useEffect, useCallback, useRef } from "react";
import { Copy } from "lucide-react";

/** Small copy action for selected transcript text. */
export function SelectionToolbar() {
  const [selectedText, setSelectedText] = useState("");
  const [position, setPosition] = useState<{ top: number; left: number } | null>(null);
  const toolbarRef = useRef<HTMLDivElement>(null);

  const handleMouseUp = useCallback(() => {
    const selection = window.getSelection();
    const text = selection?.toString().trim();
    if (!text || text.length < 2) return;
    const range = selection?.getRangeAt(0);
    if (!range) return;
    const rect = range.getBoundingClientRect();
    setSelectedText(text);
    setPosition({ top: rect.top - 40, left: rect.left + rect.width / 2 });
  }, []);

  useEffect(() => {
    document.addEventListener("mouseup", handleMouseUp);
    return () => document.removeEventListener("mouseup", handleMouseUp);
  }, [handleMouseUp]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (toolbarRef.current && !toolbarRef.current.contains(event.target as Node)) {
        setPosition(null);
      }
    };
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setPosition(null);
    };
    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, []);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(selectedText);
    setPosition(null);
  };

  if (!position) return null;

  return (
    <div
      ref={toolbarRef}
      className="fixed z-[9999]"
      style={{ top: position.top, left: position.left, transform: "translateX(-50%)" }}
    >
      <button
        onClick={handleCopy}
        aria-label="Copy selected text"
        className="flex items-center gap-1.5 rounded-lg border border-border/40 bg-popover px-2.5 py-1.5 text-xs text-muted-foreground shadow-xl transition-colors hover:bg-accent hover:text-foreground"
      >
        <Copy className="h-3 w-3" />
        Copy
      </button>
    </div>
  );
}
