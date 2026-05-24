import { create } from "zustand";
import * as api from "@/api";

interface VaultState {
  state: "uninitialized" | "locked" | "unlocked";
  isLoading: boolean;
  error: string | null;
  checkStatus: () => Promise<void>;
  init: (password: string) => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  restoreSession: () => Promise<boolean>;
  clearError: () => void;
}

export const useVaultStore = create<VaultState>((set) => ({
  state: "locked",
  isLoading: false,
  error: null,

  checkStatus: async () => {
    try {
      const status = await api.getVaultStatus();
      set({ state: status.state as VaultState["state"] });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  init: async (password: string) => {
    set({ isLoading: true, error: null });
    try {
      await api.initVault(password);
      set({ state: "unlocked", isLoading: false });
    } catch (err) {
      set({ error: String(err), isLoading: false });
      throw err;
    }
  },

  unlock: async (password: string) => {
    set({ isLoading: true, error: null });
    try {
      await api.unlockVault(password);
      set({ state: "unlocked", isLoading: false });
    } catch (err) {
      set({ error: String(err), isLoading: false });
      throw err;
    }
  },

  lock: async () => {
    try {
      await api.lockVault();
      set({ state: "locked" });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  restoreSession: async () => {
    try {
      const restored = await api.tryRestoreSession();
      if (restored) {
        set({ state: "unlocked" });
      }
      return restored;
    } catch {
      return false;
    }
  },

  clearError: () => set({ error: null }),
}));
