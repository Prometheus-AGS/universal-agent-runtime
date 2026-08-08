"use client";

import { Autocomplete } from "@base-ui/react/autocomplete";
import { CheckIcon, SearchIcon } from "lucide-react";
import * as React from "react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  InputGroup,
  InputGroupAddon,
} from "@/components/ui/input-group";
import { cn } from "@/lib/utils";

type CommandFilterContextValue = {
  matches: (value: string) => boolean;
};

const CommandFilterContext = React.createContext<CommandFilterContextValue | null>(null);

function commandItemValues(children: React.ReactNode): string[] {
  const values: string[] = [];

  React.Children.forEach(children, (child) => {
    if (!React.isValidElement<{ children?: React.ReactNode; value?: unknown }>(child)) {
      return;
    }

    if (child.type === CommandItem && typeof child.props.value === "string") {
      values.push(child.props.value);
    }
    values.push(...commandItemValues(child.props.children));
  });

  return values;
}

function Command({
  className,
  children,
  ...props
}: React.ComponentProps<"div">) {
  const items = React.useMemo(() => commandItemValues(children), [children]);
  const [query, setQuery] = React.useState("");
  const { contains } = Autocomplete.useFilter();
  const filterContext = React.useMemo<CommandFilterContextValue>(
    () => ({ matches: (value) => contains(value, query) }),
    [contains, query],
  );

  return (
    <CommandFilterContext.Provider value={filterContext}>
      <Autocomplete.Root
        inline
        open
        items={items}
        value={query}
        onValueChange={setQuery}
        filter={contains}
        autoHighlight="always"
        keepHighlight
      >
        <div
          data-slot="command"
          className={cn(
            "flex size-full flex-col overflow-hidden rounded-xl! bg-popover p-1 text-popover-foreground",
            className,
          )}
          {...props}
        >
          {children}
        </div>
      </Autocomplete.Root>
    </CommandFilterContext.Provider>
  );
}

function CommandDialog({
  title = "Command Palette",
  description = "Search for a command to run...",
  children,
  className,
  showCloseButton = false,
  ...props
}: Omit<React.ComponentProps<typeof Dialog>, "children"> & {
  title?: string;
  description?: string;
  className?: string;
  showCloseButton?: boolean;
  children: React.ReactNode;
}) {
  return (
    <Dialog {...props}>
      <DialogHeader className="sr-only">
        <DialogTitle>{title}</DialogTitle>
        <DialogDescription>{description}</DialogDescription>
      </DialogHeader>
      <DialogContent
        className={cn(
          "top-1/3 translate-y-0 overflow-hidden rounded-xl! p-0",
          className,
        )}
        showCloseButton={showCloseButton}
      >
        <Command>{children}</Command>
      </DialogContent>
    </Dialog>
  );
}

function CommandInput({
  className,
  ...props
}: React.ComponentProps<typeof Autocomplete.Input>) {
  return (
    <div data-slot="command-input-wrapper" className="p-1 pb-0">
      <InputGroup className="h-8! rounded-lg! border-input/30 bg-input/30 shadow-none! *:data-[slot=input-group-addon]:pl-2!">
        <Autocomplete.Input
          data-slot="command-input"
          className={cn(
            "w-full text-sm outline-hidden disabled:cursor-not-allowed disabled:opacity-50",
            className,
          )}
          {...props}
        />
        <InputGroupAddon>
          <SearchIcon className="size-4 shrink-0 opacity-50" />
        </InputGroupAddon>
      </InputGroup>
    </div>
  );
}

function CommandList({
  className,
  ...props
}: React.ComponentProps<typeof Autocomplete.List>) {
  return (
    <Autocomplete.List
      data-slot="command-list"
      className={cn(
        "no-scrollbar max-h-72 scroll-py-1 overflow-x-hidden overflow-y-auto outline-none",
        className,
      )}
      {...props}
    />
  );
}

function CommandEmpty({
  className,
  ...props
}: React.ComponentProps<typeof Autocomplete.Empty>) {
  return (
    <Autocomplete.Empty
      data-slot="command-empty"
      className={cn("py-6 text-center text-sm", className)}
      {...props}
    />
  );
}

type CommandGroupProps = Omit<
  React.ComponentProps<typeof Autocomplete.Group>,
  "children"
> & {
  children?: React.ReactNode;
  heading?: React.ReactNode;
};

function CommandGroup({
  className,
  children,
  heading,
  ...props
}: CommandGroupProps) {
  const filterContext = React.useContext(CommandFilterContext);
  const values = React.useMemo(() => commandItemValues(children), [children]);
  if (filterContext && !values.some(filterContext.matches)) {
    return null;
  }

  return (
    <Autocomplete.Group
      data-slot="command-group"
      className={cn(
        "overflow-hidden p-1 text-foreground **:[[data-slot=command-group-heading]]:px-2 **:[[data-slot=command-group-heading]]:py-1.5 **:[[data-slot=command-group-heading]]:text-xs **:[[data-slot=command-group-heading]]:font-medium **:[[data-slot=command-group-heading]]:text-muted-foreground",
        className,
      )}
      {...props}
    >
      {heading !== undefined && (
        <Autocomplete.GroupLabel data-slot="command-group-heading">
          {heading}
        </Autocomplete.GroupLabel>
      )}
      {children}
    </Autocomplete.Group>
  );
}

function CommandSeparator({
  className,
  ...props
}: React.ComponentProps<typeof Autocomplete.Separator>) {
  return (
    <Autocomplete.Separator
      data-slot="command-separator"
      className={cn("-mx-1 h-px w-auto bg-border", className)}
      {...props}
    />
  );
}

type CommandItemProps = Omit<
  React.ComponentProps<typeof Autocomplete.Item>,
  "onSelect"
> & {
  onSelect?: (value: string) => void;
};

function CommandItem({
  className,
  children,
  onClick,
  onSelect,
  value,
  ...props
}: CommandItemProps) {
  const filterContext = React.useContext(CommandFilterContext);
  if (
    filterContext
    && typeof value === "string"
    && !filterContext.matches(value)
  ) {
    return null;
  }

  return (
    <Autocomplete.Item
      data-slot="command-item"
      className={cn(
        "group/command-item relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none in-data-[slot=dialog-content]:rounded-lg! data-[disabled]:pointer-events-none data-[disabled]:opacity-50 data-[highlighted]:bg-muted data-[highlighted]:text-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 data-[highlighted]:**:[svg]:text-foreground",
        className,
      )}
      value={value}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) {
          onSelect?.(typeof value === "string" ? value : "");
        }
      }}
      {...props}
    >
      {children}
      <CheckIcon className="ml-auto opacity-0 group-has-data-[slot=command-shortcut]/command-item:hidden group-data-[checked=true]/command-item:opacity-100" />
    </Autocomplete.Item>
  );
}

function CommandShortcut({
  className,
  ...props
}: React.ComponentProps<"span">) {
  return (
    <span
      data-slot="command-shortcut"
      className={cn(
        "ml-auto text-xs tracking-widest text-muted-foreground group-data-[highlighted]/command-item:text-foreground",
        className,
      )}
      {...props}
    />
  );
}

export {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
};
