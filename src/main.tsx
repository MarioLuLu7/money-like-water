import React from "react";
import ReactDOM from "react-dom/client";
import { Theme } from "@radix-ui/themes";
import App from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import "@radix-ui/themes/styles.css";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <Theme appearance="dark" accentColor="green" grayColor="slate" radius="small" scaling="95%">
        <App />
      </Theme>
    </ErrorBoundary>
  </React.StrictMode>,
);
