import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { importKeys, exportKeys } from "@/api";
import { useUIStore } from "@/store";
import { Upload, Download, FileText } from "lucide-react";
import type { Environment } from "@/types";

const FORMATS = ["csv", "json", "dotenv"];
const ENVIRONMENTS: Environment[] = ["Production", "Staging", "Development", "Testing"];

export function ImportExportPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [importFormat, setImportFormat] = useState("csv");
  const [importContent, setImportContent] = useState("");
  const [importEnv, setImportEnv] = useState<Environment>("Development");
  const [exportFormat, setExportFormat] = useState("csv");
  const [exportEnv, setExportEnv] = useState<Environment | "">("");
  const [exportResult, setExportResult] = useState<string | null>(null);

  const importMutation = useMutation({
    mutationFn: () => importKeys(importFormat, importContent, importEnv),
    onSuccess: (result) => {
      addNotification("success", `导入完成: ${result.imported} 条成功`);
      setImportContent("");
    },
    onError: (err) => addNotification("error", `导入失败: ${err}`),
  });

  const exportMutation = useMutation({
    mutationFn: () => exportKeys(exportFormat, exportEnv || undefined),
    onSuccess: (content) => { setExportResult(content); addNotification("success", "导出成功"); },
    onError: (err) => addNotification("error", `导出失败: ${err}`),
  });

  const handleDownload = () => {
    if (!exportResult) return;
    const blob = new Blob([exportResult], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `keys-export.${exportFormat === "dotenv" ? "env" : exportFormat}`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const inputStyle = { background: "var(--bg-surface)", border: "1px solid var(--border-default)", color: "var(--text-primary)", outline: "none" };
  const selectClass = "w-full px-3 py-2.5 rounded-lg text-sm appearance-none transition-smooth";

  return (
    <div className="space-y-5 animate-fade-in">
      <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>导入导出</h2>

      <div className="grid grid-cols-2 gap-5">
        {/* Import */}
        <div className="p-5 rounded-xl space-y-4" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
          <h3 className="text-sm font-semibold flex items-center gap-2" style={{ color: "var(--text-primary)" }}>
            <Upload size={14} /> 导入
          </h3>
          <div>
            <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>格式</label>
            <select value={importFormat} onChange={(e) => setImportFormat(e.target.value)} className={selectClass} style={inputStyle}>
              {FORMATS.map((f) => <option key={f} value={f}>{f.toUpperCase()}</option>)}
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>目标环境</label>
            <select value={importEnv} onChange={(e) => setImportEnv(e.target.value as Environment)} className={selectClass} style={inputStyle}>
              {ENVIRONMENTS.map((env) => <option key={env} value={env}>{env}</option>)}
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>内容</label>
            <textarea value={importContent} onChange={(e) => setImportContent(e.target.value)} rows={6} className="w-full px-3 py-2.5 rounded-lg text-sm font-mono transition-smooth" style={inputStyle} placeholder={`粘贴 ${importFormat.toUpperCase()} 内容...`} />
          </div>
          <button onClick={() => importMutation.mutate()} disabled={!importContent || importMutation.isPending} className="w-full py-2.5 rounded-lg text-sm font-medium transition-smooth" style={{ background: "var(--brand-primary)", color: "white", opacity: !importContent || importMutation.isPending ? 0.5 : 1 }}>
            {importMutation.isPending ? "导入中..." : "导入"}
          </button>
        </div>

        {/* Export */}
        <div className="p-5 rounded-xl space-y-4" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
          <h3 className="text-sm font-semibold flex items-center gap-2" style={{ color: "var(--text-primary)" }}>
            <Download size={14} /> 导出
          </h3>
          <div>
            <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>格式</label>
            <select value={exportFormat} onChange={(e) => setExportFormat(e.target.value)} className={selectClass} style={inputStyle}>
              {FORMATS.map((f) => <option key={f} value={f}>{f.toUpperCase()}</option>)}
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>环境筛选</label>
            <select value={exportEnv} onChange={(e) => setExportEnv(e.target.value as Environment | "")} className={selectClass} style={inputStyle}>
              <option value="">所有环境</option>
              {ENVIRONMENTS.map((env) => <option key={env} value={env}>{env}</option>)}
            </select>
          </div>
          <button onClick={() => exportMutation.mutate()} disabled={exportMutation.isPending} className="w-full py-2.5 rounded-lg text-sm font-medium transition-smooth" style={{ background: "var(--status-success)", color: "white" }}>
            {exportMutation.isPending ? "导出中..." : "导出"}
          </button>
          {exportResult && (
            <div className="space-y-3 animate-fade-in">
              <textarea value={exportResult} readOnly rows={6} className="w-full px-3 py-2.5 rounded-lg text-sm font-mono" style={inputStyle} />
              <button onClick={handleDownload} className="w-full flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-medium transition-smooth" style={{ background: "var(--bg-surface-hover)", color: "var(--text-secondary)", border: "1px solid var(--border-default)" }}>
                <FileText size={14} /> 下载文件
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
