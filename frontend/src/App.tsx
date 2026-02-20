import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TopNav } from "@/components/layout/top-nav";
import { ChatPage } from "@/pages/chat-page";
import { AdminPage } from "@/pages/admin-page";
import { AboutPage } from "@/pages/about-page";
import { Toaster } from "sonner";
import { useThreadHydration } from "@/stores/thread-registry-store";

const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 30_000, retry: 1 } },
});

function AppRoutes() {
  // Hydrate thread list from PGlite once on mount (DB is guaranteed ready here)
  useThreadHydration();
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background">
      <TopNav />
      <Routes>
        {/* Chat — activeThreadId from store drives which thread is shown */}
        <Route
          path="/threads"
          element={
            <div className="flex flex-1 overflow-hidden">
              <ChatPage />
            </div>
          }
        />
        {/* Admin — full width, inner sidebar inside AdminPage */}
        <Route
          path="/admin/*"
          element={
            <div className="flex flex-1 overflow-hidden">
              <AdminPage />
            </div>
          }
        />
        {/* About — full width */}
        <Route
          path="/about"
          element={
            <div className="flex flex-1 overflow-hidden">
              <AboutPage />
            </div>
          }
        />
        <Route path="*" element={<Navigate to="/threads" replace />} />
      </Routes>
      <Toaster position="bottom-right" richColors />
    </div>
  );
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AppRoutes />
      </BrowserRouter>
    </QueryClientProvider>
  );
}
