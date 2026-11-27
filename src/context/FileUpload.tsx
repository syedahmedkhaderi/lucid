import { useCallback, useEffect, useRef, useState } from "react";
import { FilePlus2, Loader2 } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { addFilesToWorkspace, supportedPaths } from "../workspace/addMaterial";
import { cn } from "../lib/utils";
import { useWorkspaceStore } from "../stores/workspaceStore";
import { InfoTip } from "../ui";

type DragState = { dragging: boolean };

/**
 * Files dropped on the window, from Tauri's own drag-drop event. (The browser's
 * `File.path` is empty in WebView2, so the old drop zone never received a path.)
 * Paths arrive whole; unsupported ones are reported, not silently dropped.
 */
export function useTauriFileDrop(
  onPaths: (paths: string[], skipped: string[]) => void,
  enabled = true,
): DragState {
  const [dragging, setDragging] = useState(false);
  const handler = useRef(onPaths);
  handler.current = onPaths;

  useEffect(() => {
    if (!enabled) return;
    let stop: (() => void) | undefined;
    let cancelled = false;
    import("@tauri-apps/api/webview")
      .then(({ getCurrentWebview }) =>
        getCurrentWebview().onDragDropEvent((event) => {
          const p = event.payload;
          if (p.type === "enter" || p.type === "over") setDragging(true);
          else if (p.type === "leave") setDragging(false);
          else if (p.type === "drop") {
            setDragging(false);
            const { ok, skipped } = supportedPaths(p.paths);
            handler.current(ok, skipped);
          }
        }),
      )
      .then((fn) => {
        if (cancelled) fn();
        else stop = fn;
      })
      .catch(() => {
        /* no Tauri (browser preview): browsing still works */
      });
    return () => {
      cancelled = true;
      stop?.();
    };
  }, [enabled]);

  return { dragging };
}

/** Says what a drop or a pick did, in one line (null when nothing needs saying). */
export function describeAdd(added: number, failed: number, skipped: number): string | null {
  const parts: string[] = [];
  if (added > 0) parts.push(`${added} ${added === 1 ? "document was" : "documents were"} added`);
  if (failed > 0) parts.push(`${failed} couldn't be read`);
  if (skipped > 0) parts.push(`${skipped} ${skipped === 1 ? "isn't" : "aren't"} a PDF, text, Markdown, Word, PowerPoint or CSV file`);
  return parts.length ? parts.join(" · ") + "." : "Nothing was added.";
}

/**
 * "Choose files": adds PDF, text, Markdown, Word, PowerPoint and CSV files to the workspace
 * (library and workspace together). Dropping files on the window does the same
 * (see `useTauriFileDrop`, wired where the window's content lives).
 */
export function FileUpload({ workspaceId: given, className, compact = false, actionLabel = "Add documents" }: { workspaceId?: string; className?: string; compact?: boolean; actionLabel?: string }) {
  const openId = useWorkspaceStore((s) => s.openId);
  const workspaceId = given ?? openId;
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const browse = useCallback(async () => {
    setMessage(null);
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "Context files", extensions: ["pdf", "txt", "md", "docx", "pptx", "csv"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const { ok, skipped } = supportedPaths(paths);
      setBusy(true);
      const result = await addFilesToWorkspace(ok, workspaceId);
      const summary = describeAdd(result.added, result.failed.length, skipped.length);
      setMessage(result.failed[0]?.error ? `${result.failed[0].error}${summary ? ` · ${summary}` : ""}` : summary);
    } catch (e) {
      setMessage(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [workspaceId]);

  useEffect(() => {
    if (!message) return;
    const t = setTimeout(() => setMessage(null), 6000);
    return () => clearTimeout(t);
  }, [message]);

  const hint = "Up to 10 documents, 50 MB each. PDF, text, Markdown, Word, PowerPoint or CSV.";
  return (
    <div className={cn("w-full", className)}>
      <div className={compact ? "flex items-center gap-1" : undefined}>
        <button
          type="button"
          onClick={browse}
          disabled={busy}
          aria-busy={busy}
          className={cn(
            "ui-body flex w-full items-center gap-2 rounded-lg border border-dashed border-border text-left text-muted-foreground outline-none transition-colors hover:border-foreground/30 hover:bg-accent/50 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/60 disabled:opacity-60",
            compact ? "min-w-0 flex-1 px-2.5 py-1.5" : "px-3 py-2.5",
          )}
        >
          {busy ? <Loader2 className="h-4 w-4 shrink-0 animate-spin" aria-hidden="true" /> : <FilePlus2 className="h-4 w-4 shrink-0" aria-hidden="true" />}
          <span className="min-w-0">
            <span className="block font-medium text-foreground">{busy ? "Adding documents…" : actionLabel}</span>
            {!compact && <span className="ui-meta block">{hint}</span>}
          </span>
        </button>
        {compact && <InfoTip label="About adding files">{hint}</InfoTip>}
      </div>
      {message && (
        <p role="status" className="ui-meta mt-1.5 text-muted-foreground">
          {message}
        </p>
      )}
    </div>
  );
}
