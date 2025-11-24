import { invoke } from "@tauri-apps/api/core"
import { createContext, useContext, useEffect, useState, useCallback } from "react";
import type { VintageProgramData } from "@/lib/types";

interface ManifestContextType {
  manifest: VintageProgramData | null
  refreshManifest: () => Promise<void>
}

const ManifestContext = createContext<ManifestContextType | null>(null);

export const ManifestProvider = ({ children }: { children: React.ReactNode }) => {
    const [manifest, setManifest] = useState<VintageProgramData | null>(null);

    const refreshManifest = useCallback(async () => {
        try {
            const data = await invoke<VintageProgramData>("get_manifest_info");
            setManifest(data);
        } catch (err) {
            console.error("Failed to fetch manifest:", err);
        }
    }, []);

    useEffect(() => {
        refreshManifest();
    }, [refreshManifest]);

    return (
        <ManifestContext.Provider value={{ manifest, refreshManifest }}>
            {children}
        </ManifestContext.Provider>
    );
};

export const useManifest = () => {
    const context = useContext(ManifestContext);
    if (!context) {
        throw new Error("useManifest must be used within ManifestProvider");
    }
    return context;
};