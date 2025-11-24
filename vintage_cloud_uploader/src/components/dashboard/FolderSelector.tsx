import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Label } from "@/components/ui/label"
import { Folder, Plus } from "lucide-react"
import { Button } from "@/components/ui/button"
import { useEffect, useState } from "react"
import { useManifest } from "@/components/context/manifestContext"
import { Input } from "../ui/input"

import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
  } from "@/components/ui/dialog"

interface FolderSelectorProps {
  selectedFolder: string
  onFolderChange: (folder: string) => void
}

export function FolderSelector({ selectedFolder, onFolderChange }: FolderSelectorProps) {
  const [folderWorlds, setFolderWorlds] = useState<string[]>([])
  const { manifest } = useManifest()

  const [newFolderName, setNewFolderName] = useState<string>("")

  const handleNewFolderNameChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setNewFolderName(event.target.value)
  }

  const handleAddNewFolder = () => {
      setFolderWorlds([...folderWorlds, newFolderName])
      setNewFolderName("")
  }

  useEffect(() => {
    if (manifest) {
      const worlds = Object.keys(manifest.all_file_info)
      setFolderWorlds(worlds)
      // Set default selection if none selected and worlds available
      if (!selectedFolder && worlds.length > 0) {
        onFolderChange(worlds[0])
      }
    }
  }, [manifest])

  return (
    <div className="grid gap-2">
      <Label htmlFor="world-select" className="text-sm font-medium">
        Select World Folder
      </Label>
      <div className="w-full flex justify-center gap-2">
        <Select value={selectedFolder} onValueChange={onFolderChange}>
          <SelectTrigger id="world-select" className="w-full">
            <div className="flex items-center gap-2">
              <Folder className="h-4 w-4 text-muted-foreground" />
              <SelectValue placeholder="Select world..." />
            </div>
          </SelectTrigger>
          <SelectContent>
            {folderWorlds.map((world) => (
              <SelectItem key={world} value={world}>
                {world}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Dialog>
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-6 w-6" />
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add New Folder</DialogTitle>
            </DialogHeader>
            <Input type="text" placeholder="Folder Name" value={newFolderName} onChange={handleNewFolderNameChange} />
            <DialogFooter>
              <DialogClose asChild>
                <Button variant="outline">Cancel</Button>
              </DialogClose>
              <Button onClick={handleAddNewFolder}>Add</Button>
            </DialogFooter>
          </DialogContent>
          </Dialog>
      </div>
      <p className="text-xs text-muted-foreground">
        Choose the Vintage Story save folder you want to back up.
      </p>
    </div>
  )
}

