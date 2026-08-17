import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { CreateEnvironmentDialog } from "./components/CreateEnvironmentDialog";
import { Sidebar } from "./components/Sidebar";
import type { NavPage } from "./components/Sidebar";
import { EnvironmentDetailPage } from "./pages/EnvironmentDetailPage";
import { EnvironmentsListPage } from "./pages/EnvironmentsListPage";
import { HooksPage } from "./pages/HooksPage";
import { SettingsPage } from "./pages/SettingsPage";

const queryClient = new QueryClient();

function App() {
  const [navPage, setNavPage] = useState<NavPage>("environments");
  const [selected, setSelected] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);

  function handleNav(page: NavPage) {
    setNavPage(page);
    if (page === "environments") {
      setSelected(null);
    }
  }

  function handleQuickCreate() {
    setNavPage("environments");
    setSelected(null);
    setShowCreate(true);
  }

  return (
    <QueryClientProvider client={queryClient}>
      <div className="flex h-screen w-screen overflow-hidden bg-neutral-950 text-neutral-100">
        <Sidebar active={navPage} onNav={handleNav} onQuickCreate={handleQuickCreate} />
        <main className="min-w-0 flex-1 overflow-y-auto">
          {navPage === "hooks" && <HooksPage onBack={() => handleNav("environments")} />}
          {navPage === "settings" && <SettingsPage onBack={() => handleNav("environments")} />}
          {navPage === "environments" &&
            (selected === null ? (
              <EnvironmentsListPage onSelect={setSelected} />
            ) : (
              <EnvironmentDetailPage name={selected} onBack={() => setSelected(null)} />
            ))}
        </main>
      </div>
      {showCreate && navPage === "environments" && selected === null && (
        <CreateEnvironmentDialog onClose={() => setShowCreate(false)} />
      )}
    </QueryClientProvider>
  );
}

export default App;
