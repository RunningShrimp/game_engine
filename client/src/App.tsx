import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import NotFound from "@/pages/NotFound";
import { Route, Switch } from "wouter";
import ErrorBoundary from "./components/ErrorBoundary";
import { ThemeProvider } from "./contexts/ThemeContext";
import EditorLayout from "./components/EditorLayout";
import Welcome from "./pages/Welcome";
import SceneEditor from "./pages/SceneEditor";
import AssetBrowser from "./pages/AssetBrowser";
import EntityManager from "./pages/EntityManager";
import DebugTools from "./pages/DebugTools";
import Settings from "./pages/Settings";
import Documentation from "./pages/Documentation";


function Router() {
  return (
    <Switch>
      <Route path="/" component={Welcome} />
      <Route path="/scene">
        <EditorLayout>
          <SceneEditor />
        </EditorLayout>
      </Route>
      <Route path="/assets">
        <EditorLayout>
          <AssetBrowser />
        </EditorLayout>
      </Route>
      <Route path="/entities">
        <EditorLayout>
          <EntityManager />
        </EditorLayout>
      </Route>
      <Route path="/debug">
        <EditorLayout>
          <DebugTools />
        </EditorLayout>
      </Route>
      <Route path="/settings">
        <EditorLayout>
          <Settings />
        </EditorLayout>
      </Route>
      <Route path="/docs">
        <EditorLayout>
          <Documentation />
        </EditorLayout>
      </Route>
      <Route path="/404" component={NotFound} />
      {/* Final fallback route */}
      <Route component={NotFound} />
    </Switch>
  );
}

// NOTE: About Theme
// - First choose a default theme according to your design style (dark or light bg), than change color palette in index.css
//   to keep consistent foreground/background color across components
// - If you want to make theme switchable, pass `switchable` ThemeProvider and use `useTheme` hook

function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider
        defaultTheme="dark"
        // switchable
      >
        <TooltipProvider>
          <Toaster />
          <Router />
        </TooltipProvider>
      </ThemeProvider>
    </ErrorBoundary>
  );
}

export default App;
