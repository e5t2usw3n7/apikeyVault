import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import { Notification } from "@/components/ui/Notification";

export function MainLayout() {
  return (
    <div className="flex h-screen bg-vault-bg">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
        <StatusBar />
      </div>
      <Notification />
    </div>
  );
}
