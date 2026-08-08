import { type FC, useState } from "react";
import { ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useSettingsTypesMeta } from "../model/use-settings-types-meta";
import { GenericSchemaPanel } from "./generic-schema-panel";
import {
  CATEGORIES,
  NAV_ITEMS,
} from "./settings-navigation";
import { PANEL_MAP } from "./settings-panel-registry";

export const SettingsPage: FC = () => {
  const [active, setActive] = useState<string>("provider");
  const types = useSettingsTypesMeta();

  const availableKeys = new Set(types.map((t) => t.key));
  const customPanelKeys = new Set(["prompt_caching", "user_settings"]);
  const activeItem = NAV_ITEMS.find((item) => item.key === active);

  return (
    <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
      {/* Sidebar */}
      <aside className="flex shrink-0 flex-col border-b border-border bg-card md:w-52 md:border-b-0 md:border-r">
        <div className="border-b border-border px-4 py-3">
          <p className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">
            Settings
          </p>
        </div>
        <nav className="flex flex-col overflow-y-auto p-2">
          {CATEGORIES.map((cat) => {
            const items = NAV_ITEMS.filter((n) => n.category === cat);
            return (
              <div key={cat} className="mb-1">
                <p className="px-3 pb-1 pt-2 font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground">
                  {cat}
                </p>
                {items.map(({ key, label, subtitle, icon: Icon }) => {
                  const available =
                    availableKeys.size === 0 ||
                    availableKeys.has(key) ||
                    customPanelKeys.has(key);
                  return (
                    <Button
                      key={key}
                      onClick={() => setActive(key)}
                      disabled={!available}
                      variant={active === key ? "secondary" : "ghost"}
                      size="sm"
                      className={cn(
                        "flex h-auto w-full items-center justify-start gap-2.5 py-2 text-left font-medium",
                        active === key
                          ? "text-foreground"
                          : "text-muted-foreground hover:text-foreground",
                        !available && "cursor-not-allowed opacity-40",
                      )}
                      title={subtitle}
                    >
                      <Icon
                        size={13}
                        className={cn(
                          "shrink-0",
                          active === key
                            ? "text-primary"
                            : "text-muted-foreground",
                        )}
                      />
                      <span className="truncate text-xs">{label}</span>
                      {active === key && (
                        <ChevronRight
                          size={11}
                          className="ml-auto shrink-0 text-muted-foreground"
                        />
                      )}
                    </Button>
                  );
                })}
              </div>
            );
          })}
        </nav>
      </aside>

      {/* Content */}
      <section aria-label="Settings detail" className="flex min-w-0 flex-1 flex-col overflow-hidden">
        {PANEL_MAP[active]?.() ??
          (activeItem ? (
            <GenericSchemaPanel
              namespace={active}
              title={activeItem.label}
              subtitle={activeItem.subtitle}
            />
          ) : (
            <div className="flex flex-1 items-center justify-center">
              <p className="font-mono text-xs text-muted-foreground">
                Select a settings category
              </p>
            </div>
          ))}
      </section>
    </div>
  );
};
