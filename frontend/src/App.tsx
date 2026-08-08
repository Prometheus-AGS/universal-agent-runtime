import { lazy, Suspense } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router";
import { AppShell } from "@/app/shell/app-shell";
import { Titlebar } from "@/components/Titlebar";
import { ChatPage } from "@/pages/chat-page";
import { Toaster } from "sonner";
import { OfflineBanner } from "@/components/OfflineBanner";
import { useAppBootstrap } from "@/hooks/use-app-bootstrap";

const LazyAdminPage = lazy(() => import("@/pages/admin-page").then((module) => ({
  default: module.AdminPage,
})));
const LazyAboutPage = lazy(() => import("@/pages/about-page").then((module) => ({
  default: module.AboutPage,
})));

export function RouteLoadingFallback({ label }: { label: string }) {
  return (
    <div
      className="flex flex-1 items-center justify-center bg-background p-6 text-center"
      role="status"
      aria-live="polite"
    >
      <span className="font-mono text-xs text-fg-sub">Loading {label}…</span>
    </div>
  );
}

export function AppRoutes() {
  // Hydrate thread list from PGlite once on mount (DB is guaranteed ready here).
  useAppBootstrap();
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background">
      <Titlebar />
      <OfflineBanner />
      <AppShell>
        <Routes>
          {/* Chat — activeThreadId from store drives which thread is shown */}
          <Route
            path="/threads"
            element={
              <div className="flex h-full flex-1 overflow-hidden">
                <ChatPage />
              </div>
            }
          />
          {/* Configuration surfaces — full width inside the shared application shell */}
          <Route
            path="/admin/*"
            element={
              <div className="flex h-full flex-1 overflow-hidden">
                <Suspense fallback={<RouteLoadingFallback label="administration" />}>
                  <LazyAdminPage />
                </Suspense>
              </div>
            }
          />
          {/* About — full width */}
          <Route
            path="/about"
            element={
              <div className="flex h-full flex-1 overflow-hidden">
                <Suspense fallback={<RouteLoadingFallback label="about" />}>
                  <LazyAboutPage />
                </Suspense>
              </div>
            }
          />
          <Route path="*" element={<Navigate to="/threads" replace />} />
        </Routes>
      </AppShell>
      <Toaster position="bottom-right" richColors />
    </div>
  );
}

export function App() {
  return (
    <BrowserRouter>
      <AppRoutes />
    </BrowserRouter>
  );
}
