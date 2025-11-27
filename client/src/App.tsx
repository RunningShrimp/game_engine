import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import NotFound from "@/pages/NotFound";
import { Route, Switch } from "wouter";
import ErrorBoundary from "./components/ErrorBoundary";
import { ThemeProvider } from "./contexts/ThemeContext";
import { WebSocketProvider } from "./contexts/WebSocketContext";
import { UndoRedoProvider } from "./contexts/UndoRedoContext";
import { ProjectProvider } from "./contexts/ProjectContext";
import EditorLayout from "./components/EditorLayout";
import Welcome from "./pages/Welcome";
import SceneEditor from "./pages/SceneEditor";
import AssetBrowser from "./pages/AssetBrowser";
import EntityManager from "./pages/EntityManager";
import DebugTools from "./pages/DebugTools";
import Settings from "./pages/Settings";
import Documentation from "./pages/Documentation";
import CodeEditor from "./pages/CodeEditor";
import ShaderEditor from "./pages/ShaderEditor";
import PCGTools from "./pages/PCGTools";
import AIAssistant from "./pages/AIAssistant";
import BlueprintEditor from "./pages/BlueprintEditor";
import AnimationBlueprint from "./pages/AnimationBlueprint";
import AnimationStateMachine from "./pages/AnimationStateMachine";
import PhysicsAnimation from "./pages/PhysicsAnimation";
import PluginManager from "./pages/PluginManager";
import MotionCapture from "./pages/MotionCapture";
import FacialAnimation from "./pages/FacialAnimation";
import AnimationRetargeting from "./pages/AnimationRetargeting";
import BehaviorTree from "./pages/BehaviorTree";
import DialogueSystem from "./pages/DialogueSystem";


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
      <Route path="/code">
        <EditorLayout>
          <CodeEditor />
        </EditorLayout>
      </Route>
      <Route path="/shader">
        <EditorLayout>
          <ShaderEditor />
        </EditorLayout>
      </Route>
      <Route path="/pcg">
        <EditorLayout>
          <PCGTools />
        </EditorLayout>
      </Route>
      <Route path="/ai">
        <EditorLayout>
          <AIAssistant />
        </EditorLayout>
      </Route>
      <Route path="/blueprint">
        <EditorLayout>
          <BlueprintEditor />
        </EditorLayout>
      </Route>
      <Route path="/animation">
        <EditorLayout>
          <AnimationBlueprint />
        </EditorLayout>
      </Route>
      <Route path="/statemachine">
        <EditorLayout>
          <AnimationStateMachine />
        </EditorLayout>
      </Route>
      <Route path="/physics">
        <EditorLayout>
          <PhysicsAnimation />
        </EditorLayout>
      </Route>
      <Route path="/plugins">
        <EditorLayout>
          <PluginManager />
        </EditorLayout>
      </Route>
      <Route path="/mocap">
        <EditorLayout>
          <MotionCapture />
        </EditorLayout>
      </Route>
      <Route path="/facial">
        <EditorLayout>
          <FacialAnimation />
        </EditorLayout>
      </Route>
      <Route path="/retarget">
        <EditorLayout>
          <AnimationRetargeting />
        </EditorLayout>
      </Route>
      <Route path="/behavior">
        <EditorLayout>
          <BehaviorTree />
        </EditorLayout>
      </Route>
      <Route path="/dialogue">
        <EditorLayout>
          <DialogueSystem />
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
        <WebSocketProvider>
          <UndoRedoProvider>
            <ProjectProvider>
              <TooltipProvider>
                <Toaster />
                <Router />
              </TooltipProvider>
            </ProjectProvider>
          </UndoRedoProvider>
        </WebSocketProvider>
      </ThemeProvider>
    </ErrorBoundary>
  );
}

export default App;
