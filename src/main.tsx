import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import "./styles.css";

startApplication();

function startApplication() {
  renderApplication(findRootElement());
}

function renderApplication(rootElement: HTMLElement) {
  createRoot(rootElement).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

function findRootElement(): HTMLElement {
  const rootElement = document.getElementById("root");

  if (!rootElement) {
    throw new Error("DRAFT root element is missing");
  }

  return rootElement;
}
