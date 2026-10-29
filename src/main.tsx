import React from "react";
import ReactDOM from "react-dom/client";
import PanelPage from "./app/panel/page";
import "./styles/globals.css";
import * as stylex from "@stylexjs/stylex";
import { applyColors, applyTheme } from "@/lib/apply-theme";
import { resolveLang, setLang } from "@/lib/locale";
import { page } from "@/lib/themes";
import { getSettingsService, onSettingsChanged } from "@/services/settings";
import { getWindowLabelService } from "@/services/windows";
import type { Settings } from "@/types/services/settings";

document.body.classList.add(...(stylex.props(page).className?.split(' ') ?? []))
setLang(navigator.language.toLowerCase().startsWith('tr') ? 'tr' : 'en')

const apply = (s: Settings) => {
  applyTheme(s.theme)
  setLang(resolveLang(s.language, navigator.language))
  applyColors(s.customColors)
}

await getSettingsService().then(apply, console.error)
const label = getWindowLabelService()
const Root = label === 'settings' ? (await import("./app/settings/page")).default : label === 'preview' ? (await import("./app/preview/page")).default : PanelPage
const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement)
const render = () =>
  root.render(
    <React.StrictMode>
      <Root key={Root === PanelPage ? document.documentElement.lang : undefined} />
    </React.StrictMode>,
  );
render()
onSettingsChanged((s) => {
  apply(s)
  render()
})
