import { invoke } from "@tauri-apps/api/core"
import { createContext, useContext, useState, useCallback, ReactNode } from "react"
import { useManifest } from "./manifestContext"

interface UploadContextType {
  isUploading: boolean
  isDownloading: boolean
  error: string | null
  upload: (folderName: string) => Promise<void>
  download: (folderName: string) => Promise<void>
}

const UploadContext = createContext<UploadContextType | null>(null)

export const UploadProvider = ({ children }: { children: ReactNode }) => {
  const [isUploading, setIsUploading] = useState(false)
  const [isDownloading, setIsDownloading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const { refreshManifest } = useManifest()

  const upload = useCallback(async (folderName: string) => {
    setIsUploading(true)
    setError(null)
    try {
      await invoke("run_upload", { folderBucket: folderName })
      // Refresh manifest after successful upload
      await refreshManifest()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
      throw err
    } finally {
      setIsUploading(false)
    }
  }, [refreshManifest])

  const download = useCallback(async (folderName: string) => {
    setIsDownloading(true)
    setError(null)
    try {
      await invoke("run_download", { folderBucket: folderName })
      // Refresh manifest after successful download
      await refreshManifest()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
      throw err
    } finally {
      setIsDownloading(false)
    }
  }, [refreshManifest])

  return (
    <UploadContext.Provider value={{ isUploading, isDownloading, error, upload, download }}>
      {children}
    </UploadContext.Provider>
  )
}

export const useUpload = () => {
  const context = useContext(UploadContext)
  if (!context) {
    throw new Error("useUpload must be used within UploadProvider")
  }
  return context
}

