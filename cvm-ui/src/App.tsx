import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { EnvironmentDetailPage } from "./pages/EnvironmentDetailPage";
import { EnvironmentsListPage } from "./pages/EnvironmentsListPage";

const queryClient = new QueryClient();

function App() {
  const [selected, setSelected] = useState<string | null>(null);

  return (
    <QueryClientProvider client={queryClient}>
      <main className="min-h-screen bg-neutral-950 text-neutral-100">
        {selected === null ? (
          <EnvironmentsListPage onSelect={setSelected} />
        ) : (
          <EnvironmentDetailPage name={selected} onBack={() => setSelected(null)} />
        )}
      </main>
    </QueryClientProvider>
  );
}

export default App;
