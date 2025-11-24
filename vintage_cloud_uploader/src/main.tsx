import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ManifestProvider } from "./components/context/manifestContext";
import { UploadProvider } from "./components/context/uploadContext";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ManifestProvider>
      <UploadProvider>
        <App />
      </UploadProvider>
    </ManifestProvider>
  </React.StrictMode>,
);
