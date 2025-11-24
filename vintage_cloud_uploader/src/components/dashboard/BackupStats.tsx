import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { Database, Clock, HardDrive } from "lucide-react"
import { useManifest } from "@/components/context/manifestContext"
import { useEffect, useState } from "react"


export function BackupStats() {

  const { manifest } = useManifest()



  if (!manifest) {
    return null
  }

  const [totalBackups, setTotalBackups] = useState(0)
  const [spaceUsed, setSpaceUsed] = useState(0)
  const [lastSync, setLastSync] = useState(0)

  useEffect(() => {
    if (manifest) {
      setTotalBackups(Object.keys(manifest.all_file_info).length)
      const spaceUsed = Object.values(manifest.all_file_info).reduce((acc, curr) => acc + Object.values(curr.files).reduce((acc, curr) => acc + (curr.file_size ?? 0), 0), 0)
      setSpaceUsed(Math.round(spaceUsed / 1024 / 1024 / 1024))
      setLastSync(manifest.last_opened)
    }
  }, [manifest])

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Storage Stats</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="rounded-md bg-primary/10 p-2 text-primary">
              <Database className="h-4 w-4" />
            </div>
            <div className="space-y-0.5">
              <p className="text-sm font-medium">Total Backups</p>
              <p className="text-xs text-muted-foreground">Across all worlds</p>
            </div>
          </div>
          <div className="font-bold">{totalBackups}</div>
        </div>

        <Separator />

        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="rounded-md bg-primary/10 p-2 text-primary">
              <HardDrive className="h-4 w-4" />
            </div>
            <div className="space-y-0.5">
              <p className="text-sm font-medium">Space Used</p>
              <p className="text-xs text-muted-foreground">Cloud storage</p>
            </div>
          </div>
          <div className="font-bold">{spaceUsed} GB</div>
        </div>

        <Separator />

        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="rounded-md bg-primary/10 p-2 text-primary">
              <Clock className="h-4 w-4" />
            </div>
            <div className="space-y-0.5">
              <p className="text-sm font-medium">Last Sync</p>
              <p className="text-xs text-muted-foreground">Automatic backup</p>
            </div>
          </div>
          <div className="font-bold">{lastSync} ago</div>
        </div>
      </CardContent>
    </Card>
  )
}

