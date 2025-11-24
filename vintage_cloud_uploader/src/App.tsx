import { useState } from "react"
import { UploadProgress } from "@/components/dashboard/UploadProgress"
import { FolderSelector } from "@/components/dashboard/FolderSelector"
import { BackupStats } from "@/components/dashboard/BackupStats"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Cloud, Settings } from "lucide-react"
import { Button } from "@/components/ui/button"

export default function App() {
  const [selectedFolder, setSelectedFolder] = useState<string>("")

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      {/* Header */}
      <header className="sticky top-0 z-10 border-b bg-background/95 px-6 py-4 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto flex max-w-5xl items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="rounded-lg bg-primary p-2 text-primary-foreground">
              <Cloud className="h-5 w-5" />
            </div>
            <div>
              <h1 className="text-lg font-bold leading-none">Vintage Cloud</h1>
              <p className="text-xs text-muted-foreground">Save Game Uploader</p>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <ScrollArea className="flex-1">
        <main className="mx-auto max-w-5xl p-6">
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            {/* Left Column - Controls */}
            <div className="space-y-6 lg:col-span-2">
              {/* Folder Selection Card */}
              <div className="rounded-xl border bg-card p-6 shadow-sm">
                <FolderSelector
                  selectedFolder={selectedFolder}
                  onFolderChange={setSelectedFolder}
                />
              </div>

              {/* Progress Section */}
              <UploadProgress selectedFolder={selectedFolder} />
            </div>

            {/* Right Column - Stats */}
            <div className="space-y-6">
              <BackupStats />
            </div>
          </div>
        </main>
      </ScrollArea>
    </div>
  )
}
