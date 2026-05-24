import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { importKeys, exportKeys } from "@/api";
import { useUIStore } from "@/store";
import type { Environment } from "@/types";

const FORMATS = ["csv", "json", "dotenv"];
const ENVIRONMENTS: Environment[] = [
  "Production",
  "Staging",
  "Development",
  "Testing",
];

export function ImportExportPage() {
  const addNotification = useUIStore((s) => s.addNotification);

  // Import state
  const [importFormat, setImportFormat] = useState("csv");
  const [importContent, setImportContent] = useState("");
  const [importEnv, setImportEnv] = useState<Environment>("Development");

  // Export state
  const [exportFormat, setExportFormat] = useState("csv");
  const [exportEnv, setExportEnv] = useState<Environment | "">("");
  const [exportResult, setExportResult] = useState<string | null>(null);

  const importMutation = useMutation({
    mutationFn: () => importKeys(importFormat, importContent, importEnv),
    onSuccess: (result) => {
      addNotification(
        "success",
        `导入完成: ${result.imported} 条成功, ${result.skipped} 条跳过`
      );
      if (result.errors.length > 0) {
        result.errors.forEach((err) => addNotification("error", err));
      }
      setImportContent("");
    },
    onError: (err) => addNotification("error", `导入失败: ${err}`),
  });

  const exportMutation = useMutation({
    mutationFn: () => exportKeys(exportFormat, exportEnv || undefined),
    onSuccess: (content) => {
      setExportResult(content);
      addNotification("success", "导出成功");
    },
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

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-vault-text">导入导出</h2>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 导入 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <h3 className="text-lg font-semibold text-vault-text mb-4">导入</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-vault-muted mb-2">
                格式
              </label>
              <select
                value={importFormat}
                onChange={(e) => setImportFormat(e.target.value)}
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
              >
                {FORMATS.map((f) => (
                  <option key={f} value={f}>
                    {f.toUpperCase()}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-vault-muted mb-2">
                目标环境
              </label>
              <select
                value={importEnv}
                onChange={(e) => setImportEnv(e.target.value as Environment)}
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
              >
                {ENVIRONMENTS.map((env) => (
                  <option key={env} value={env}>
                    {env}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-vault-muted mb-2">
                内容
              </label>
              <textarea
                value={importContent}
                onChange={(e) => setImportContent(e.target.value)}
                rows={8}
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text font-mono text-sm focus:outline-none focus:border-vault-primary"
                placeholder={`粘贴 ${importFormat.toUpperCase()} 内容...`}
              />
            </div>

            <button
              onClick={() => importMutation.mutate()}
              disabled={!importContent || importMutation.isPending}
              className="w-full py-2 bg-vault-primary hover:bg-vault-primary/80 text-white rounded-lg disabled:opacity-50"
            >
              {importMutation.isPending ? "导入中..." : "导入"}
            </button>
          </div>
        </div>

        {/* 导出 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <h3 className="text-lg font-semibold text-vault-text mb-4">导出</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-vault-muted mb-2">
                格式
              </label>
              <select
                value={exportFormat}
                onChange={(e) => setExportFormat(e.target.value)}
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
              >
                {FORMATS.map((f) => (
                  <option key={f} value={f}>
                    {f.toUpperCase()}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-vault-muted mb-2">
                环境筛选（可选）
              </label>
              <select
                value={exportEnv}
                onChange={(e) => setExportEnv(e.target.value as Environment | "")}
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
              >
                <option value="">所有环境</option>
                {ENVIRONMENTS.map((env) => (
                  <option key={env} value={env}>
                    {env}
                  </option>
                ))}
              </select>
            </div>

            <button
              onClick={() => exportMutation.mutate()}
              disabled={exportMutation.isPending}
              className="w-full py-2 bg-vault-success hover:bg-vault-success/80 text-white rounded-lg disabled:opacity-50"
            >
              {exportMutation.isPending ? "导出中..." : "导出"}
            </button>

            {exportResult && (
              <div className="space-y-3">
                <textarea
                  value={exportResult}
                  readOnly
                  rows={8}
                  className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text font-mono text-sm"
                />
                <button
                  onClick={handleDownload}
                  className="w-full py-2 bg-vault-border hover:bg-vault-border/80 text-vault-text rounded-lg"
                >
                  下载文件
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
