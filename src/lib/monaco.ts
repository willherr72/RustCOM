import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";

// Tell Monaco how to spawn workers in a Vite-bundled context.
(self as unknown as { MonacoEnvironment: { getWorker: () => Worker } }).MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};

// Define a "zed-cool" theme matching our app.
monaco.editor.defineTheme("zed-cool", {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "comment", foreground: "6a6d72", fontStyle: "italic" },
    { token: "string", foreground: "8fc4a8" },
    { token: "keyword", foreground: "7a86c8" },
    { token: "number", foreground: "c4a78c" },
  ],
  colors: {
    "editor.background": "#1c1d20",
    "editor.foreground": "#d4d6d9",
    "editorLineNumber.foreground": "#6a6d72",
    "editorCursor.foreground": "#7a86c8",
    "editor.selectionBackground": "#2a2c30",
    "editor.lineHighlightBackground": "#1f2124",
  },
});

export { monaco };
