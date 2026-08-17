import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { EnvironmentDetailPage } from "./pages/EnvironmentDetailPage";
import { EnvironmentsListPage } from "./pages/EnvironmentsListPage";
import { HooksPage } from "./pages/HooksPage";

const queryClient = new QueryClient();

function App() {
  const [selected, setSelected] = useState<string | null>(null);
  const [showHooks, setShowHooks] = useState(false);

  return (
    <QueryClientProvider client={queryClient}>
      <main className="min-h-screen bg-neutral-950 text-neutral-100">
        {showHooks ? (
          <HooksPage onBack={() => setShowHooks(false)} />
        ) : selected === null ? (
          <EnvironmentsListPage onSelect={setSelected} onOpenHooks={() => setShowHooks(true)} />
        ) : (
          <EnvironmentDetailPage name={selected} onBack={() => setSelected(null)} />
        )}
      </main>
    </QueryClientProvider>
  );
}

export default App;
