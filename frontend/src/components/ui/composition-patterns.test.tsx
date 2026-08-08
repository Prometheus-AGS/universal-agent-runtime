import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import {
  AssistantRuntimeProvider,
  ComposerPrimitive,
  type ChatModelAdapter,
  useLocalRuntime,
} from "@assistant-ui/react"
import { beforeEach, describe, expect, test, vi } from "vitest"

import { BreadcrumbLink } from "@/components/ui/breadcrumb"
import { Button } from "@/components/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Sidebar,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"

const mobileViewport = vi.hoisted(() => ({ value: false }))

vi.mock("@/hooks/use-mobile", () => ({
  useIsMobile: () => mobileViewport.value,
}))

beforeEach(() => {
  mobileViewport.value = false
})

function AssistantActionHarness({
  adapter,
  onClick,
}: {
  adapter: ChatModelAdapter
  onClick: () => void
}) {
  const runtime = useLocalRuntime(adapter)

  return (
    <AssistantRuntimeProvider runtime={runtime}>
      <ComposerPrimitive.Root>
        <ComposerPrimitive.Input aria-label="Message" />
        <ComposerPrimitive.Send
          render={<Button data-testid="composed-send" onClick={onClick} />}
        >
          Send
        </ComposerPrimitive.Send>
      </ComposerPrimitive.Root>
    </AssistantRuntimeProvider>
  )
}

describe("Base UI composition patterns", () => {
  test("preserves Button and BreadcrumbLink semantics through composed anchors", () => {
    render(
      <>
        <Button render={<a href="/docs" />} nativeButton={false}>
          Documentation
        </Button>
        <BreadcrumbLink render={<a href="/runs" />}>Runs</BreadcrumbLink>
      </>
    )

    const composedButton = screen.getByRole("button", {
      name: "Documentation",
    })
    expect(composedButton.tagName).toBe("A")
    expect(composedButton).toHaveAttribute("href", "/docs")
    expect(screen.getByRole("link", { name: "Runs" })).toHaveAttribute(
      "href",
      "/runs"
    )
  })

  test("toggles the desktop sidebar between expanded and collapsed", async () => {
    const user = userEvent.setup()
    render(
      <SidebarProvider>
        <Sidebar>Navigation</Sidebar>
        <SidebarTrigger />
      </SidebarProvider>
    )

    const sidebar = document.querySelector(
      '[data-slot="sidebar"][data-state]'
    )
    expect(sidebar).toHaveAttribute("data-state", "expanded")

    await user.click(screen.getByRole("button", { name: "Toggle Sidebar" }))
    expect(sidebar).toHaveAttribute("data-state", "collapsed")
  })

  test("opens the mobile sidebar through the shared trigger", async () => {
    mobileViewport.value = true
    const user = userEvent.setup()
    render(
      <SidebarProvider>
        <Sidebar>Mobile navigation</Sidebar>
        <SidebarTrigger />
      </SidebarProvider>
    )

    expect(screen.queryByText("Mobile navigation")).not.toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: "Toggle Sidebar" }))

    expect(await screen.findByText("Mobile navigation")).toBeVisible()
    expect(screen.getByText("Displays the mobile sidebar.")).toBeInTheDocument()
  })

  test("selects an option with keyboard navigation", async () => {
    const user = userEvent.setup()
    const onValueChange = vi.fn()
    render(
      <Select defaultValue="auto" onValueChange={onValueChange}>
        <SelectTrigger aria-label="Approval mode">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="auto">Auto</SelectItem>
          <SelectItem value="ask">Ask</SelectItem>
          <SelectItem value="deny">Deny</SelectItem>
        </SelectContent>
      </Select>
    )

    const trigger = screen.getByRole("combobox", { name: "Approval mode" })
    trigger.focus()
    await user.keyboard("{ArrowDown}")
    await user.keyboard("{ArrowDown}{Enter}")

    await waitFor(() => {
      expect(onValueChange).toHaveBeenCalledWith("ask", expect.anything())
    })
  })

  test("merges assistant action behavior onto one render element", async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    const run = vi.fn(async function* () {})

    render(<AssistantActionHarness adapter={{ run }} onClick={onClick} />)

    const send = screen.getByRole("button", { name: "Send" })
    expect(send).toBe(screen.getByTestId("composed-send"))
    expect(send).toBeDisabled()

    await user.type(screen.getByRole("textbox", { name: "Message" }), "Hello")
    await waitFor(() => expect(send).toBeEnabled())
    await user.click(send)

    expect(onClick).toHaveBeenCalledOnce()
    await waitFor(() => expect(run).toHaveBeenCalledOnce())
  })
})
