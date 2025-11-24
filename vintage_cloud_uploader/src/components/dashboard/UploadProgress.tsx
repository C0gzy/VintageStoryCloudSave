import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { Badge } from "@/components/ui/badge"
import { UploadCloud, DownloadCloud, AlertCircle } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  ButtonGroup,
  ButtonGroupSeparator,
} from "@/components/ui/button-group"
import { useUpload } from "@/components/context/uploadContext"
import { Alert, AlertDescription } from "@/components/ui/alert"

interface UploadProgressProps {
  selectedFolder: string
}

export function UploadProgress({ selectedFolder }: UploadProgressProps) {
  const { isUploading, isDownloading, error, upload, download } = useUpload()

  const handleUpload = async () => {
    if (!selectedFolder) {
      return
    }
    try {
      await upload(selectedFolder)
    } catch (err) {
      // Error is handled by context
      console.error("Upload failed:", err)
    }
  }

  const handleDownload = async () => {
    if (!selectedFolder) {
      return
    }
    try {
      await download(selectedFolder)
    } catch (err) {
      // Error is handled by context
      console.error("Download failed:", err)
    }
  }

  const isProcessing = isUploading || isDownloading

  return (
    <Card>
      {isProcessing ? (
        <>
          <CardHeader className="pb-4">
            <div className="flex items-center justify-between">
              <CardTitle className="text-lg">
                {isUploading ? "Upload Status" : "Download Status"}
              </CardTitle>
              <Badge variant="secondary" className="gap-1">
                {isUploading ? (
                  <UploadCloud className="h-3 w-3" />
                ) : (
                  <DownloadCloud className="h-3 w-3" />
                )}
                In Progress
              </Badge>
            </div>
            <CardDescription>
              {isUploading
                ? `Backing up "${selectedFolder}" to Cloud`
                : `Downloading "${selectedFolder}" from Cloud`}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">
                  {isUploading ? "Uploading files..." : "Downloading files..."}
                </span>
                <span className="font-medium">Processing...</span>
              </div>
              <Progress value={undefined} className="h-2" />
            </div>
          </CardContent>
        </>
      ) : (
        <>
          <CardHeader className="pb-4">
            <div className="flex items-center justify-between">
              <CardTitle className="text-lg">Cloud Save</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <ButtonGroup>
              <Button
                onClick={handleUpload}
                disabled={!selectedFolder || isProcessing}
              >
                <UploadCloud className="h-4 w-4" />
                Upload
              </Button>
              <ButtonGroupSeparator />
              <Button
                onClick={handleDownload}
                disabled={!selectedFolder || isProcessing}
              >
                <DownloadCloud className="h-4 w-4" />
                Download
              </Button>
            </ButtonGroup>
            {!selectedFolder && (
              <p className="text-xs text-muted-foreground">
                Please select a folder first
              </p>
            )}
          </CardContent>
        </>
      )}
    </Card>
  )
}

