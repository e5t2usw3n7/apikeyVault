import { useEffect } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { useVaultStore, useUIStore } from "@/store";
import { MainLayout } from "@/components/layout/MainLayout";
import { LoginPage } from "@/pages/LoginPage";
import { DashboardPage } from "@/pages/DashboardPage";
import { KeyListPage } from "@/pages/KeyListPage";
import { KeyDetailPage } from "@/pages/KeyDetailPage";
import { KeyEditPage } from "@/pages/KeyEditPage";
import { GroupListPage } from "@/pages/GroupListPage";
import { GroupEditPage } from "@/pages/GroupEditPage";
import { AuditLogPage } from "@/pages/AuditLogPage";
import { ImportExportPage } from "@/pages/ImportExportPage";
import { SettingsPage } from "@/pages/SettingsPage";

export default function App() {
  const { state, checkStatus, restoreSession } = useVaultStore();
  const theme = useUIStore((s) => s.theme);

  useEffect(() => {
    document.documentElement.classList.toggle("light", theme === "light");
  }, [theme]);

  useEffect(() => {
    const init = async () => {
      try {
        const restored = await restoreSession();
        if (!restored) {
          await checkStatus();
        }
      } catch {
        await checkStatus();
      }
    };
    init();
  }, [checkStatus, restoreSession]);

  if (state !== "unlocked") {
    return <LoginPage />;
  }

  return (
    <Routes>
      <Route element={<MainLayout />}>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/keys" element={<KeyListPage />} />
        <Route path="/keys/new" element={<KeyEditPage />} />
        <Route path="/keys/:name/:env" element={<KeyDetailPage />} />
        <Route path="/keys/:name/:env/edit" element={<KeyEditPage />} />
        <Route path="/groups" element={<GroupListPage />} />
        <Route path="/groups/new" element={<GroupEditPage />} />
        <Route path="/groups/:id/edit" element={<GroupEditPage />} />
        <Route path="/audit" element={<AuditLogPage />} />
        <Route path="/import-export" element={<ImportExportPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
