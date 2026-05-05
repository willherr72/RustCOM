import { mount } from "svelte";
import "./lib/theme.css";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error("Missing #app root element");
}

mount(App, { target });
